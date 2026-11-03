#[path = "engine_planet.rs"]
mod planet;
#[path = "engine_planet_view.rs"]
mod planet_view;
#[path = "engine_portable.rs"]
mod portable;
use crate::{
    model::*,
    noise::{MAX_NOISES, MAX_NOISES_PER_TICK, Noise, NoiseOrigin},
    physics::{Input, Simulation},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::VecDeque;

pub const MAX_TICKS: usize = 36_000;
const RECENT_MANUAL_NOISES: usize = 128;
/// Leave room for configured relative deadlines before numeric exhaustion.
pub const MAX_RUNTIME_TICK: u64 = u64::MAX - 4 * u32::MAX as u64;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    #[default]
    Recorded,
    Continuous,
}
#[derive(Default)]
struct Admission {
    ability_counts: Vec<u8>,
    control_touched: Vec<u64>,
    ability: u64,
    population: u64,
    control: u64,
    noise: u64,
    control_tick: u64,
    controls_at_tick: usize,
}
impl Admission {
    fn new(actors: usize) -> Self {
        Self {
            ability_counts: vec![0; actors],
            control_touched: vec![u64::MAX; actors],
            ..Default::default()
        }
    }
}
pub const MAX_SCENE_EDITS: usize = 512;

/// Authoring-only operations. Runtime control and I/O cannot be nested in a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum SceneEdit {
    SetWind {
        document: std::sync::Arc<crate::wind::Document>,
    },
    SetParticles {
        document: std::sync::Arc<crate::particles::Document>,
    },
    SetBotany {
        recipes: Vec<crate::botany::Recipe>,
    },
    SetCharacterBinding {
        binding: Box<crate::character_editor::Binding>,
    },
    SetMeshCollision {
        instance: String,
        enabled: bool,
    },
    CompileDistantMeshes,
    ClearDistantMeshes,
    SetNpcCapacity {
        capacity: u16,
    },
    SetAvatarAudio {
        audio: crate::npc_presentation::Audio,
    },
    SetInteractions {
        document: crate::interactions::Document,
    },
    Plant {
        edit: crate::plants::Edit,
    },
    Mesh {
        edit: crate::meshes::Edit,
    },
    Upsert {
        entity: Box<Entity>,
    },
    Remove {
        id: String,
    },
    SetCamera {
        camera: Camera,
    },
    SetLight {
        light: Light,
    },
    SetNpcGenerators {
        generators: Vec<crate::generators::Config>,
    },
    SetNpcTemplates {
        templates: Vec<Entity>,
    },
    SetAbilityCatalog {
        catalog: crate::abilities::Catalog,
    },
    SetSpatialNetwork {
        network: crate::mobility::Network,
    },
    SetNavigationGeneration {
        recipe: Option<crate::npc_navigation_generation::Recipe>,
    },
    SetPassages {
        passages: Vec<crate::passages::Config>,
    },
    SetFactionRelations {
        relations: Vec<crate::targeting::Relation>,
    },
}

fn default_session_capacity() -> usize {
    crate::targeting::MAX_PLAYERS
}

/// One typed contract, used by native controls, the CLI and MCP.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Wind {
        operation: Box<crate::wind::Operation>,
    },
    Particles {
        operation: Box<crate::particles::Operation>,
    },
    Material {
        operation: Box<crate::materials::library::Operation>,
    },
    Botany {
        operation: Box<crate::botany::Operation>,
    },
    InspectCharacterSource {
        path: String,
    },
    OpenCharacterPanel {
        open: bool,
        #[serde(default)]
        prefix: Option<String>,
    },
    InspectCharacterEditor,
    SetCharacterHandleView {
        view: crate::character_editor::HandleView,
    },
    PreviewCharacterHandle {
        prefix: String,
        handle: String,
        offset: [f32; 3],
        symmetry: bool,
        reference: crate::mesh_topology::Reference,
    },
    SetCharacterHandle {
        prefix: String,
        handle: String,
        /// Offset from the source's neutral handle, in local metres.
        offset: [f32; 3],
        symmetry: bool,
        reference: crate::mesh_topology::Reference,
        #[serde(default)]
        gesture: Option<String>,
    },
    LoadCharacterEditorSource {
        prefix: String,
    },
    UpgradeCharacterSource {
        prefix: String,
        path: String,
        reference: crate::mesh_topology::Reference,
    },
    ConnectCharacterSource {
        path: String,
        #[serde(default)]
        configuration: crate::meshes::character::Configuration,
        placement: crate::meshes::character::Placement,
    },
    SetCharacterParameters {
        prefix: String,
        configuration: crate::meshes::character::Configuration,
        #[serde(default)]
        gesture: Option<String>,
    },
    RegenerateCharacter {
        prefix: String,
    },
    SaveCharacterConfiguration {
        prefix: String,
        path: String,
    },
    LoadCharacterConfiguration {
        path: String,
    },
    PublishCharacter {
        path: String,
        #[serde(default)]
        configuration: crate::meshes::character::Configuration,
        placement: crate::meshes::character::Placement,
    },
    PublishTerrainRegion {
        publication: Box<crate::terrain::region::Publication>,
    },
    InspectMeshCollisions,
    SavePortableSave {
        path: String,
    },
    LoadPortableSave {
        path: String,
        #[serde(default)]
        mode: Option<ExecutionMode>,
    },
    InspectPortableSave {
        #[serde(default)]
        path: Option<String>,
    },
    MigratePortableSave {
        source_path: String,
        path: String,
    },
    SaveCheckpoint {
        path: String,
    },
    LoadCheckpoint {
        path: String,
        #[serde(default)]
        mode: Option<ExecutionMode>,
    },
    InspectCheckpoint,
    PlanWorldRegions {
        query: Box<crate::world_regions::Query>,
    },
    QueryTerrain {
        query: Box<crate::terrain::Query>,
    },
    QueryLandform {
        query: Box<crate::landforms::Query>,
    },
    PublishLandform {
        publication: Box<crate::landforms::Publication>,
    },
    SaveLandform {
        path: String,
        document: Box<crate::landforms::Document>,
    },
    LoadLandform {
        path: String,
    },
    PlanTerrainCoverage {
        query: Box<crate::terrain::coverage::Query>,
    },
    PublishTerrain {
        publication: Box<crate::terrain::Publication>,
    },
    SaveTerrain {
        path: String,
        document: Box<crate::terrain::Document>,
    },
    LoadTerrain {
        path: String,
    },
    CapturePlanetReference {
        document: Box<crate::planet::Document>,
        view: crate::planet::view::View,
        path: String,
        width: u32,
        height: u32,
    },
    QueryPlanet {
        query: Box<crate::planet::Query>,
    },
    PublishPlanet {
        publication: Box<crate::planet::mesh::Publication>,
    },
    SavePlanet {
        path: String,
        document: Box<crate::planet::Document>,
    },
    LoadPlanet {
        path: String,
    },
    QueryWorldSpace {
        query: crate::world_space::Query,
    },
    SetNpcCapacity {
        capacity: u16,
    },
    SetAvatarAudio {
        audio: crate::npc_presentation::Audio,
    },
    InspectNpcPresentation,
    SetNpcAudioOutput {
        enabled: bool,
        listener: [f32; 3],
        radius: f32,
    },
    InspectNpcAudioOutput,
    CaptureNpcAudio {
        path: String,
        frames: u32,
        listener: [f32; 3],
        radius: f32,
    },
    Interact {
        actor: String,
        point: String,
        action: String,
    },
    TalkNpc {
        actor: String,
        npc: String,
    },
    ChooseDialogue {
        actor: String,
        session: String,
        choice: String,
    },
    CloseDialogue {
        actor: String,
        session: String,
    },
    InspectInteractions,
    InspectNpcRoutines,
    InspectDialogue {
        actor: String,
    },
    SetInteractions {
        document: crate::interactions::Document,
    },
    SetMultiplayerInput {
        input: Input,
    },
    ReleaseMultiplayerInput {
        control: String,
    },
    InspectMultiplayerControls,
    SelectMultiplayerTarget {
        target: Option<crate::network::protocol::Target>,
    },
    PickMultiplayer {
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    },
    UseMultiplayerAbility {
        slot: u8,
        #[serde(default)]
        target: Option<crate::network::protocol::Target>,
        #[serde(default)]
        aim: Option<crate::abilities::Aim>,
    },
    UseMultiplayerAbilityAt {
        slot: u8,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        #[serde(default)]
        plane_y: Option<f32>,
    },
    CancelMultiplayerAbility,
    SetMultiplayerView {
        camera: Camera,
    },
    CaptureMultiplayer {
        path: String,
        width: u32,
        height: u32,
    },
    StartMultiplayerHost {
        address: String,
        #[serde(default = "default_session_capacity")]
        capacity: usize,
    },
    StopMultiplayerHost,
    InspectMultiplayerHost,
    ConnectMultiplayer {
        address: String,
        certificate_der: String,
    },
    ReconnectMultiplayer,
    DisconnectMultiplayer,
    InspectMultiplayerClient {
        #[serde(default)]
        actors: bool,
    },
    SendMultiplayerMovement {
        movement: crate::network::protocol::Movement,
    },
    SendMultiplayerOperation {
        operation: crate::network::protocol::Operation,
    },
    ModelEditorMesh {
        instance: String,
        reference: crate::mesh_topology::Reference,
        operation: crate::mesh_modeling::Operation,
        #[serde(default)]
        preview: bool,
    },
    SetEditorTransform {
        settings: crate::editor_transform::Settings,
    },
    InspectEditorTransform {
        #[serde(default)]
        components: bool,
    },
    PreviewEditorTransform {
        target: crate::editor_transform::Target,
        reference: crate::mesh_topology::Reference,
        transform: crate::meshes::IslandTransform,
    },
    TransformEditor {
        target: crate::editor_transform::Target,
        reference: crate::mesh_topology::Reference,
        transform: crate::meshes::IslandTransform,
    },
    SetEditorMeshView {
        view: crate::editor::MeshView,
    },
    SelectEditorMeshRegion {
        instance: String,
        reference: crate::mesh_topology::Reference,
        component: crate::mesh_screen::Kind,
        min: [f32; 2],
        max: [f32; 2],
        width: u32,
        height: u32,
        #[serde(default)]
        additive: bool,
    },
    CreateEditorMesh {
        primitive: crate::mesh_tools::Primitive,
        resolution: u16,
        size: f32,
    },
    SelectEditorMesh {
        instance: Option<String>,
    },
    PickEditorMesh {
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    },
    InspectProject,
    BrowseProjectDirectory {
        query: crate::project_browser::Query,
    },
    CreateProjectFolder {
        parent: String,
        name: String,
    },
    InspectProjectLibrary,
    UseProjectLibrary {
        path: String,
    },
    RegisterProject {
        path: String,
    },
    PinProject {
        path: String,
        pinned: bool,
    },
    ForgetProject {
        path: String,
    },
    SetProjectFavorite {
        path: String,
        favorite: bool,
    },
    DiscoverProjects {
        paths: Vec<String>,
    },
    CreateProject {
        path: String,
        name: String,
        #[serde(default)]
        discard_changes: bool,
    },
    OpenProject {
        path: String,
        #[serde(default)]
        discard_changes: bool,
    },
    NewProjectScene {
        name: String,
        #[serde(default)]
        discard_changes: bool,
    },
    OpenProjectScene {
        path: String,
        #[serde(default)]
        discard_changes: bool,
    },
    SetProjectEntry {
        path: String,
    },
    CreateSceneObject {
        kind: crate::project::ObjectKind,
    },
    DuplicateSceneObject {
        id: String,
    },
    InspectEditor,
    SetViewportPerformance {
        settings: crate::viewport_performance::Settings,
    },
    InspectViewportPerformance,
    CompileMeshLods {
        path: String,
    },
    LoadMeshLods {
        path: String,
    },
    SetMeshLodSettings {
        settings: crate::mesh_lods::Settings,
    },
    InspectMeshLods,
    SelectEditorAt {
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    },
    EditEditor {
        action: crate::editor::Action,
        expected_editor_revision: Option<u64>,
    },
    SaveEditor {
        path: String,
    },
    LoadEditor {
        path: String,
    },
    CaptureEditor {
        path: String,
        width: u32,
        height: u32,
    },
    CompareImages {
        reference_path: String,
        candidate_path: String,
        mask_path: Option<String>,
        #[serde(default)]
        regions: Vec<crate::image_similarity::Region>,
        difference_path: Option<String>,
    },
    PlanMeshSweep {
        sweeps: Vec<crate::mesh_sweep::Sweep>,
    },
    PreviewMeshMaterialFinish {
        source: Box<crate::meshes::Asset>,
        plan: crate::mesh_material_finish::Plan,
        options: crate::mesh_material_finish::Options,
    },
    InspectMeshMaterialDomain {
        asset: String,
        faces: Option<Vec<u32>>,
        #[serde(default)]
        options: crate::mesh_material_domain::Options,
    },
    PreviewMeshMaterialDomain {
        source: Box<crate::meshes::Geometry>,
        faces: Option<Vec<u32>>,
        #[serde(default)]
        options: crate::mesh_material_domain::Options,
    },
    PreviewMeshSurface {
        source: Box<crate::meshes::Asset>,
        vertices: Vec<u32>,
        anchors: Vec<u32>,
        options: crate::mesh_surface::Options,
    },
    PreviewMeshRingLoft {
        source: Box<crate::meshes::Asset>,
        lofts: Vec<crate::mesh_ring_loft::Loft>,
    },
    PreviewMeshJunctionUnion {
        source: Box<crate::meshes::Asset>,
        ports: Box<[Vec<u32>; 3]>,
        root: Option<String>,
        options: crate::mesh_junction_union::Options,
    },
    InspectMeshBoundaries {
        asset: String,
    },
    PreviewMeshJunction {
        collars: Box<[crate::mesh_organic::Collar; 3]>,
        #[serde(default)]
        options: crate::mesh_organic::Options,
        #[serde(default)]
        fusion: Option<crate::mesh_organic::Fusion>,
    },
    EditPlant {
        edit: crate::plants::Edit,
    },
    InspectPlants,
    InspectPlant {
        id: String,
    },
    InspectPlantSampling {
        id: String,
    },
    InspectPlantConnections,
    PreviewPlant {
        edit: crate::plants::Edit,
    },
    EditMesh {
        edit: crate::meshes::Edit,
    },
    PreviewMeshEdit {
        edit: crate::meshes::Edit,
    },
    InspectMeshes,
    InspectMeshObjects {
        object: Option<String>,
    },
    ProjectLandmarks {
        width: u32,
        height: u32,
        camera: Option<Camera>,
        landmarks: Vec<crate::landmark_projection::Landmark>,
    },
    InspectMeshIslands {
        asset: String,
    },
    InspectMeshTopology {
        asset: String,
    },
    SelectMeshComponents {
        selection: Option<crate::mesh_topology::Selection>,
    },
    InspectMeshSelection,
    InspectMesh {
        asset: String,
    },
    InspectMeshNormals {
        instance: String,
        tick: Option<u64>,
        faces: Option<Vec<u32>>,
        #[serde(default = "default_normal_sample_limit")]
        sample_limit: u8,
    },
    InspectMeshPose {
        instance: String,
        tick: Option<u64>,
    },
    Inspect,
    InspectExecution,
    SetPlayerView {
        view: Option<crate::player_view::Config>,
    },
    InspectPlayerView,
    SetPlanetRuntime {
        config: Option<Box<crate::planet::runtime::Config>>,
    },
    LoadPlanetRuntime {
        path: String,
    },
    InspectPlanetRuntime,
    SetPlanetView {
        view: Option<crate::planet::orbit::Settings>,
    },
    InspectPlanetView,
    FramePlanetView,
    ZoomPlanetView {
        factor: f64,
    },
    LookPlanetView {
        yaw_delta_degrees: f64,
        tilt_delta_degrees: f64,
    },
    SetPlanetViewport {
        width: u32,
        height: u32,
    },
    LookPlayerView {
        yaw_delta_degrees: f32,
        pitch_delta_degrees: f32,
    },
    SetPlayerViewInput {
        input: Input,
        control: Option<String>,
    },
    SetExecutionMode {
        mode: ExecutionMode,
    },
    InspectInstances,
    InspectCharacterControllers,
    QueryGroundTraversal {
        entity: String,
        start: [f32; 3],
        end: [f32; 3],
    },
    InspectWorld,
    InspectOrigins,
    InspectPopulation,
    InspectTemplates,
    QuerySpawnSupport {
        template: String,
        position: [f32; 3],
        max_drop: f32,
    },
    QueryGeometry {
        entity: String,
        start: [f32; 3],
        end: [f32; 3],
        #[serde(default)]
        half_extent: [f32; 3],
        #[serde(default)]
        interior: bool,
    },
    InspectGenerators,
    ControlNpcGenerator {
        generator: String,
        action: crate::generator_control::Action,
    },
    SpawnActor {
        template: String,
        position: [f32; 3],
        target: Option<String>,
    },
    DespawnActor {
        actor: String,
    },
    InspectAbilities,
    InspectAttribute {
        entity: String,
        attribute: String,
        #[serde(default)]
        ability: Option<String>,
        #[serde(default)]
        tags: Vec<String>,
    },
    InspectResources,
    InspectVitality,
    ActivateAbility {
        actor: String,
        slot: u8,
        target: Option<String>,
    },
    AimAbility {
        actor: String,
        slot: u8,
        aim: crate::abilities::Aim,
    },
    AimAbilityAt {
        actor: String,
        slot: u8,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        plane_y: f32,
    },
    CancelAbility {
        actor: String,
        execution_id: u64,
    },
    InspectNavigation,
    InspectNpcs,
    InspectNpcMind,
    InspectNpcCirculation,
    InspectNpcTactics,
    InspectNpcCooperation,
    OrderNpc {
        actor: String,
        order: crate::npc_cooperation::Order,
    },
    OrderNpcGroup {
        group: u16,
        order: crate::npc_cooperation::Order,
    },
    InspectRelations,
    InspectTargeting,
    InspectKnowledge,
    InspectHearing,
    InspectObjectives,
    InspectTerritory,
    InspectPassages,
    InspectMobility,
    QueryNavigationGeneration {
        recipe: crate::npc_navigation_generation::Recipe,
    },
    SetNpcGenerators {
        generators: Vec<crate::generators::Config>,
    },
    SetNpcTemplates {
        templates: Vec<Entity>,
    },
    SetAbilityCatalog {
        catalog: crate::abilities::Catalog,
    },
    SetSpatialNetwork {
        network: crate::mobility::Network,
    },
    SetNavigationGeneration {
        recipe: Option<crate::npc_navigation_generation::Recipe>,
    },
    SetPassages {
        passages: Vec<crate::passages::Config>,
    },
    SetFactionRelations {
        relations: Vec<crate::targeting::Relation>,
    },
    InspectNoises,
    InspectFootsteps,
    InspectPatrols,
    InspectTriggers,
    InspectSelection,
    SelectEntity {
        id: Option<String>,
    },
    SelectAt {
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    },
    SetDebugView {
        npc_vision: Option<bool>,
        npc_hearing: Option<bool>,
        noise_events: Option<bool>,
        npc_routes: Option<bool>,
    },
    EmitNoise {
        origin: NoiseOrigin,
        radius: f32,
    },
    Generate {
        recipe: RoomRecipe,
    },
    EditScene {
        edits: Vec<SceneEdit>,
    },
    /// Publish editing feedback immediately; consecutive updates with the same
    /// gesture and no intervening document revision share one undo entry.
    EditSceneLive {
        edits: Vec<SceneEdit>,
        gesture: String,
    },
    PreviewSceneEdits {
        edits: Vec<SceneEdit>,
    },
    Upsert {
        entity: Box<Entity>,
    },
    Remove {
        id: String,
    },
    SetCamera {
        camera: Camera,
    },
    SetLight {
        light: Light,
    },
    Undo,
    Redo,
    Play,
    Pause,
    Reset,
    SetInput {
        input: Input,
    },
    BindControl {
        actor: String,
    },
    SetControlInput {
        control: String,
        input: Input,
    },
    ReleaseControl {
        control: String,
    },
    InspectControls,
    Step {
        ticks: u32,
    },
    Events,
    SaveScenario {
        path: String,
        scenario: Box<crate::scenario::Scenario>,
    },
    InspectScenario {
        path: String,
    },
    RunScenario {
        path: String,
        report_path: Option<String>,
        replay_path: Option<String>,
    },
    SaveScene {
        path: String,
    },
    LoadScene {
        path: String,
    },
    SaveReplay {
        path: String,
    },
    RunReplay {
        path: String,
    },
    InspectReplay {
        path: String,
        tick: Option<u64>,
    },
    CaptureReplay {
        replay_path: String,
        path: String,
        tick: Option<u64>,
        width: u32,
        height: u32,
    },
    Capture {
        path: String,
        width: u32,
        height: u32,
    },
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Optional optimistic concurrency guard for document edits.
    pub expected_revision: Option<u64>,
    pub command: Command,
}
impl From<Command> for Request {
    fn from(command: Command) -> Self {
        Self {
            expected_revision: None,
            command,
        }
    }
}

pub const REPLAY_VERSION: u32 = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    #[serde(default)]
    pub expected_wind: Vec<crate::wind::BodyState>,
    #[serde(
        default,
        with = "crate::checkpoint::baseline",
        skip_serializing_if = "Option::is_none"
    )]
    pub baseline: Option<std::sync::Arc<[u8]>>,
    #[serde(
        default,
        skip_serializing_if = "crate::npc_presentation::StateSet::is_empty"
    )]
    pub expected_presentation: crate::npc_presentation::StateSet,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    pub version: u32,
    pub engine_version: String,
    pub physics_version: String,
    pub build_id: String,
    pub platform: String,
    pub fixed_dt: f32,
    pub scene: Scene,
    pub inputs: Vec<Input>,
    pub control_events: Vec<crate::controls::Entry>,
    pub expected_controls: crate::controls::State,
    pub noises: Vec<Noise>,
    pub ability_inputs: Vec<crate::abilities::Entry>,
    pub population_inputs: Vec<crate::population::Entry>,
    pub expected_population: crate::population::State,
    #[serde(default, skip_serializing_if = "crate::interactions::State::is_empty")]
    pub expected_interactions: crate::interactions::State,
    pub expected_generators: crate::generators::State,
    pub expected_meshes: Vec<crate::meshes::Pose>,
    pub expected_instances: crate::identity::Snapshot,
    pub expected_abilities: crate::abilities::State,
    pub expected_vitality: crate::vitality::State,
    pub expected_navigation: std::collections::BTreeMap<String, crate::navigation::NavState>,
    pub expected_character_controllers:
        std::collections::BTreeMap<String, crate::character_controller::State>,
    pub expected_mobility: std::collections::BTreeMap<String, crate::mobility::State>,
    pub expected_passages: crate::passages::Runtime,
    pub expected_npcs: std::collections::BTreeMap<String, crate::perception::NpcState>,
    pub expected_footsteps: crate::footsteps::Footsteps,
    pub expected_triggers: crate::trigger::State,
    pub expected_patrols: std::collections::BTreeMap<String, crate::patrol::State>,
    pub expected_positions: std::collections::BTreeMap<String, [f32; 3]>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
pub struct DebugView {
    pub npc_vision: bool,
    pub npc_hearing: bool,
    pub noise_events: bool,
    pub npc_routes: bool,
}

struct Selected {
    id: String,
    index: u16,
}

#[path = "engine_wind.rs"]
mod wind;

#[path = "engine_particles.rs"]
mod particles;

pub struct Engine {
    baseline: Option<std::sync::Arc<[u8]>>,
    baseline_tick: u64,
    baseline_sequences: [u64; 3],
    execution_mode: ExecutionMode,
    admission: Admission,
    execution_scope: String,
    pub scene: Scene,
    pub revision: u64,
    pub simulation: Simulation,
    pub playing: bool,
    pub exit_requested: bool,
    pub input: Input,
    pub(crate) input_revision: u64,
    pub debug_view: DebugView,
    pub view_revision: u64,
    player_view: Option<crate::player_view::State>,
    player_view_serial: u64,
    planet_view: Option<crate::planet::orbit::Atlas>,
    pub particle_preview: Option<u64>,
    pub(crate) mesh_lods: std::sync::Arc<crate::mesh_lods::Document>,
    pub mesh_lod_settings: crate::mesh_lods::Settings,
    selection: Option<Selected>,
    mesh_selection: Option<crate::mesh_topology::Selection>,
    live_edit: Option<(String, u64)>,
    undo: VecDeque<Scene>,
    redo: Vec<Scene>,
    inputs: Vec<Input>,
    controls: crate::controls::Runtime,
    control_events: Vec<crate::controls::Entry>,
    recent_manual_noises: VecDeque<Noise>,
    dropped_manual_noises: u64,
    noises: Vec<Noise>,
    consumed_noises: usize,
    ability_inputs: Vec<crate::abilities::Entry>,
    consumed_ability_inputs: usize,
    population_inputs: Vec<crate::population::Entry>,
    consumed_population_inputs: usize,
}

impl Engine {
    pub fn new(mut scene: Scene) -> Result<Self> {
        scene.validate()?;
        scene.normalize();
        let simulation = Simulation::new(&scene);
        Ok(Self {
            baseline: None,
            baseline_tick: 0,
            baseline_sequences: [0; 3],
            execution_mode: ExecutionMode::Recorded,
            admission: Admission::new(simulation.actors.actors.len()),
            execution_scope: crate::identity::fresh_scope(),
            controls: crate::controls::Runtime::new(simulation.actors.actors.len()),
            control_events: Vec::new(),
            recent_manual_noises: VecDeque::new(),
            dropped_manual_noises: 0,
            scene,
            revision: 0,
            simulation,
            playing: false,
            exit_requested: false,
            input: Input::default(),
            input_revision: 0,
            debug_view: DebugView::default(),
            view_revision: 0,
            player_view: None,
            player_view_serial: 0,
            planet_view: None,
            particle_preview: None,
            mesh_lods: Default::default(),
            mesh_lod_settings: Default::default(),
            selection: None,
            mesh_selection: None,
            live_edit: None,
            undo: VecDeque::new(),
            redo: Vec::new(),
            inputs: Vec::new(),
            noises: Vec::new(),
            consumed_noises: 0,
            ability_inputs: Vec::new(),
            consumed_ability_inputs: 0,
            population_inputs: Vec::new(),
            consumed_population_inputs: 0,
        })
    }
    pub fn new_with_mode(scene: Scene, mode: ExecutionMode) -> Result<Self> {
        let mut engine = Self::new(scene)?;
        engine.set_execution_mode(mode)?;
        Ok(engine)
    }
    pub fn execution_mode(&self) -> ExecutionMode {
        self.execution_mode
    }
    fn recording(&self) -> bool {
        self.execution_mode == ExecutionMode::Recorded
    }
    fn clock_open(&self) -> bool {
        self.simulation.tick < MAX_RUNTIME_TICK
            && (!self.recording() || self.simulation.tick < MAX_TICKS as u64)
    }
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) -> Result<()> {
        if self.planet_active() && mode != ExecutionMode::Continuous {
            return Err(invalid(
                "Planet runtime currently requires continuous execution",
            ));
        }
        if mode == self.execution_mode {
            return Ok(());
        }
        if self.playing
            || self.simulation.tick != 0
            || self.admission.ability != 0
            || self.admission.population != 0
            || self.admission.control != 0
            || self.admission.noise != 0
            || self.input != Input::default()
        {
            return Err(invalid(
                "Execution mode can change only while paused before inputs and controls; reset first",
            ));
        }
        self.execution_mode = mode;
        if mode == ExecutionMode::Continuous {
            self.recent_manual_noises.reserve(RECENT_MANUAL_NOISES);
        }
        Ok(())
    }
    pub fn inspect_execution(&self) -> Value {
        json!({"mode":self.execution_mode,"replay_available":self.recording(),
            "admitted":{"controls":self.admission.control,"abilities":self.admission.ability,"population":self.admission.population,"noises":self.admission.noise},
            "retained":{"ticks":self.inputs.len(),"controls":self.control_events.len(),"abilities":self.ability_inputs.len(),"population":self.population_inputs.len(),"noises":self.noises.len()},
            "pending":{"abilities":self.ability_inputs.len()-self.consumed_ability_inputs,"population":self.population_inputs.len()-self.consumed_population_inputs,"noises":self.noises.len()-self.consumed_noises},
            "diagnostics":{"recent_manual_noises":self.recent_manual_noises.len(),"capacity":RECENT_MANUAL_NOISES,"dropped_manual_noises":self.dropped_manual_noises},
            "admission_limits":{"abilities_per_actor":crate::abilities::PER_ACTOR_PER_TICK,"abilities_per_tick":crate::abilities::input_capacity(self.simulation.actors.actors.len()),"control_budget_unit":if self.recording(){"events"}else{"touched_actors"}},
            "recording_tick_limit":MAX_TICKS})
    }
    /// A continuous run has no complete command history; never persist it as a replay.
    pub fn recorded_replay(&self) -> Result<Replay> {
        if !self.recording() {
            return Err(invalid(
                "Continuous execution has no complete replay history",
            ));
        }
        Ok(self.replay())
    }
    pub fn guard(&self, revision: Option<u64>) -> Result<()> {
        if revision.is_some_and(|r| r != self.revision) {
            return Err(Error {
                code: "revision_conflict",
                message: format!("Current scene revision is {}", self.revision),
            });
        }
        Ok(())
    }
    pub fn mesh_reference(&self) -> crate::mesh_topology::Reference {
        crate::mesh_topology::Reference {
            scope: self.execution_scope.clone(),
            revision: self.revision,
        }
    }
    fn guard_mesh_reference(&self, reference: &crate::mesh_topology::Reference) -> Result<()> {
        if reference != &self.mesh_reference() {
            return Err(Error {
                code: "topology_conflict",
                message: "Mesh component reference is stale; inspect topology again".into(),
            });
        }
        Ok(())
    }
    pub fn replace(&mut self, mut scene: Scene) -> Result<()> {
        if self.planet_active() {
            return Err(invalid(
                "Disable planet runtime before replacing authoring scene",
            ));
        }
        scene.validate()?;
        scene.normalize();
        scene.entities.sort_by(|a, b| a.id.cmp(&b.id));
        self.live_edit = None;
        if self.undo.len() == 32 {
            self.undo.pop_front();
        }
        self.undo
            .push_back(std::mem::replace(&mut self.scene, scene));
        self.redo.clear();
        self.revision += 1;
        self.reset();
        Ok(())
    }
    fn staged_edits(&self, edits: impl ExactSizeIterator<Item = SceneEdit>) -> Result<Scene> {
        if edits.len() == 0 || edits.len() > MAX_SCENE_EDITS {
            return Err(invalid("Scene edit batch requires 1..512 operations"));
        }
        let mut scene = self.scene.clone();
        let mut changed_meshes = std::collections::BTreeSet::new();
        for (index, edit) in edits.enumerate() {
            match edit {
                SceneEdit::SetWind { document } => {
                    document.validate()?;
                    scene.version = scene.version.max(crate::wind::SCENE_VERSION);
                    scene.wind = document;
                }
                SceneEdit::SetBotany { recipes } => {
                    scene.version = SCENE_VERSION;
                    scene.botany = recipes;
                }
                SceneEdit::SetCharacterBinding { binding } => {
                    binding.validate()?;
                    scene.version = scene.version.max(if binding.recipe.version >= 2 {
                        109
                    } else {
                        106
                    });
                    if let Some(old) = scene
                        .characters
                        .iter_mut()
                        .find(|b| b.recipe.placement.prefix == binding.recipe.placement.prefix)
                    {
                        *old = *binding;
                    } else {
                        scene.characters.push(*binding);
                    }
                }
                SceneEdit::Mesh { edit } => {
                    if self.playing && matches!(&edit, crate::meshes::Edit::ModelSelection { .. }) {
                        return Err(invalid("Pause simulation before component modeling"));
                    }
                    if let Some((asset, reference)) = edit.component_reference() {
                        self.guard_mesh_reference(reference)?;
                        if changed_meshes.contains(asset) {
                            return Err(invalid(
                                "Mesh source changed earlier in batch; inspect before component editing",
                            ));
                        }
                    }
                    for asset in edit.changed_assets().into_iter().flatten() {
                        changed_meshes.insert(asset.to_owned());
                    }
                    if matches!(&edit, crate::meshes::Edit::SetMaterial { material, .. } if material.surface.is_some())
                        || matches!(&edit, crate::meshes::Edit::PutAsset { asset } if asset.material.surface.is_some())
                    {
                        scene.version = scene.version.max(110);
                    }
                    if matches!(&edit, crate::meshes::Edit::SetMaterial { material, .. } if material.procedural.is_some())
                        || matches!(&edit, crate::meshes::Edit::PutAsset { asset } if asset.material.procedural.is_some())
                    {
                        scene.version = scene.version.max(112);
                    }
                    if matches!(&edit,crate::meshes::Edit::SetMaterial {material,..} if material.procedural.as_ref().is_some_and(|p|p.recipe.requires_extended_surface()))
                        || matches!(&edit,crate::meshes::Edit::PutAsset {asset} if asset.material.procedural.as_ref().is_some_and(|p|p.recipe.requires_extended_surface()))
                    {
                        scene.version = scene.version.max(crate::materials::rock::SCENE_VERSION);
                    }
                    if matches!(&edit,crate::meshes::Edit::SetMaterial {material,..} if material.procedural.as_ref().is_some_and(|p|p.recipe.marble.is_some()))
                        || matches!(&edit,crate::meshes::Edit::PutAsset {asset} if asset.material.procedural.as_ref().is_some_and(|p|p.recipe.marble.is_some()))
                    {
                        scene.version = scene.version.max(crate::materials::marble::SCENE_VERSION);
                    }
                    scene.plants.apply_mesh(edit, &mut scene.meshes)?;
                }
                SceneEdit::Plant { edit } => {
                    // A recipe can change both derived assets, including when it
                    // removes the dependency before a later component edit.
                    for connection in &scene.plants.connections {
                        for name in [&connection.parent_instance, &connection.child_instance] {
                            if let Some(instance) =
                                scene.meshes.instances.iter().find(|i| &i.id == name)
                            {
                                changed_meshes.insert(instance.asset.clone());
                            }
                        }
                    }
                    if let crate::plants::Edit::PutConnection { connection } = &edit {
                        for name in [&connection.parent_instance, &connection.child_instance] {
                            if let Some(instance) =
                                scene.meshes.instances.iter().find(|i| &i.id == name)
                            {
                                changed_meshes.insert(instance.asset.clone());
                            }
                        }
                    }
                    if let crate::plants::Edit::Put { definition, .. }
                    | crate::plants::Edit::AdoptTrajectory { definition, .. } = &edit
                    {
                        changed_meshes.insert(definition.id.clone());
                    }
                    if let crate::plants::Edit::SetAssembly { id, .. } = &edit {
                        changed_meshes.insert(id.clone());
                    }
                    scene.plants.apply(edit, &mut scene.meshes)?;
                }
                SceneEdit::Upsert { entity } => {
                    if let Some(old) = scene.entities.iter_mut().find(|e| e.id == entity.id) {
                        *old = *entity;
                    } else {
                        scene.entities.push(*entity);
                    }
                }
                SceneEdit::Remove { id } => {
                    let position =
                        scene
                            .entities
                            .iter()
                            .position(|e| e.id == id)
                            .ok_or_else(|| Error {
                                code: "not_found",
                                message: format!("Edit {index}: entity {id} does not exist"),
                            })?;
                    scene.entities.remove(position);
                }
                SceneEdit::SetMeshCollision { instance, enabled } => {
                    scene.collision_meshes.retain(|id| id != &instance);
                    if enabled {
                        scene.collision_meshes.push(instance);
                    }
                }
                SceneEdit::SetNpcCapacity { capacity } => scene.npc_capacity = capacity,
                SceneEdit::SetAvatarAudio { audio } => scene.avatar_audio = audio,
                SceneEdit::SetInteractions { document } => scene.interactions = document,
                SceneEdit::SetNpcGenerators { generators } => scene.npc_generators = generators,
                SceneEdit::SetNpcTemplates { templates } => scene.npc_templates = templates,
                SceneEdit::SetAbilityCatalog { catalog } => scene.ability_catalog = catalog,
                SceneEdit::SetCamera { camera } => scene.camera = camera,
                SceneEdit::CompileDistantMeshes => {
                    scene.distant_meshes = crate::distant::compile(&scene.meshes)?
                }
                SceneEdit::ClearDistantMeshes => scene.distant_meshes = Default::default(),
                SceneEdit::SetParticles { document } => {
                    document.validate()?;
                    scene.version =
                        scene
                            .version
                            .max(if document.sources.iter().any(|s| s.version >= 2) {
                                crate::particles::FIRE_SCENE_VERSION
                            } else {
                                crate::particles::SCENE_VERSION
                            });
                    scene.particles = document;
                }
                SceneEdit::SetLight { light } => {
                    if light.shadow.as_ref().is_some_and(|s| s.receiver_plane_bias) {
                        scene.version = scene.version.max(111);
                    }
                    scene.light = light;
                }
                SceneEdit::SetFactionRelations { relations } => scene.faction_relations = relations,
                SceneEdit::SetSpatialNetwork { network } => {
                    scene.spatial_network = network;
                    scene.navigation_generation = None;
                }
                SceneEdit::SetNavigationGeneration { recipe } => {
                    scene.navigation_generation = recipe
                }
                SceneEdit::SetPassages { passages } => scene.passages = passages,
            }
        }
        scene.plants.rebuild_connections(&mut scene.meshes)?;
        Ok(scene)
    }
    pub(crate) fn prepare_mesh_edit(
        &self,
        edit: &crate::meshes::Edit,
    ) -> Result<(Scene, Option<crate::mesh_modeling::Report>)> {
        if self.playing && matches!(edit, crate::meshes::Edit::ModelSelection { .. }) {
            return Err(invalid("Pause simulation before component modeling"));
        }
        if let Some((_, reference)) = edit.component_reference() {
            self.guard_mesh_reference(reference)?;
        }
        let mut scene = self.scene.clone();
        let report = scene
            .plants
            .apply_mesh_report(edit.clone(), &mut scene.meshes)?;
        scene.plants.rebuild_connections(&mut scene.meshes)?;
        scene.validate()?;
        Ok((scene, report))
    }
    pub(crate) fn preview_edits(&self, edits: Vec<SceneEdit>) -> Result<Scene> {
        let mut scene = self.staged_edits(edits.into_iter())?;
        scene.validate()?;
        scene.normalize();
        scene.entities.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(scene)
    }
    fn apply_edits(&mut self, edits: impl ExactSizeIterator<Item = SceneEdit>) -> Result<()> {
        let scene = self.staged_edits(edits)?;
        self.replace(scene)
    }
    pub fn reset(&mut self) {
        self.particle_preview = None;
        self.planet_view = None;
        let planet = self.simulation.planet.as_ref().map(|w| w.config.clone());
        let planet_view = planet
            .as_ref()
            .and_then(|_| self.player_view.as_ref().map(|v| v.config.clone()));
        if self
            .mesh_lods
            .packs
            .iter()
            .any(|p| !self.scene.meshes.assets.iter().any(|a| p.matches(a)))
        {
            self.mesh_lods = std::sync::Arc::new(self.mesh_lods.filtered(&self.scene.meshes));
        }
        self.player_view = None;
        self.baseline = None;
        self.baseline_tick = 0;
        self.baseline_sequences = [0; 3];
        self.live_edit = None;
        self.execution_scope = crate::identity::fresh_scope();
        if self.mesh_selection.take().is_some() {
            self.view_revision += 1;
        }
        if let Some(selected) = &mut self.selection {
            if let Some(index) = self.scene.entities.iter().position(|e| e.id == selected.id) {
                selected.index = index as u16;
            } else {
                self.selection = None;
                self.view_revision += 1;
            }
        }
        self.simulation = Simulation::new(&self.scene);
        self.controls = crate::controls::Runtime::new(self.simulation.actors.actors.len());
        self.admission = Admission::new(self.simulation.actors.actors.len());
        self.control_events.clear();
        self.recent_manual_noises.clear();
        self.dropped_manual_noises = 0;
        self.inputs.clear();
        self.noises.clear();
        self.consumed_noises = 0;
        self.ability_inputs.clear();
        self.consumed_ability_inputs = 0;
        self.population_inputs.clear();
        self.consumed_population_inputs = 0;
        self.input = Input::default();
        self.input_revision += 1;
        self.playing = false;
        if let Some(config) = planet {
            self.enable_planet_runtime(config)
                .expect("previously validated planet reset");
            if let Some(view) = planet_view {
                self.execute(Command::SetPlayerView { view: Some(view) }.into())
                    .expect("validated planet view");
            }
        }
    }
    pub fn validate_input(input: Input) -> Result<()> {
        if !input.x.is_finite() || !input.z.is_finite() || input.x.abs() > 1. || input.z.abs() > 1.
        {
            return Err(invalid("Input axes must be finite and in [-1,1]"));
        }
        Ok(())
    }
    pub fn advance(&mut self, ticks: u32) -> Result<()> {
        if ticks == 0
            || ticks > 3600
            || self
                .simulation
                .tick
                .checked_add(ticks as u64)
                .is_none_or(|t| t > MAX_RUNTIME_TICK)
            || self.recording() && self.inputs.len() + ticks as usize > MAX_TICKS
        {
            return Err(invalid(
                "Step requires1..3600ticks, available clock and recording budget",
            ));
        }
        for _ in 0..ticks {
            if let Err(error) = self.simulation.prepare_planet_step() {
                self.playing = false;
                self.input = Input::default();
                self.input_revision += 1;
                return Err(error);
            }
            let controls = self
                .controls
                .inputs(&self.simulation.actors, &self.simulation.vitality);
            self.simulation.step_with_controls(
                self.input,
                controls,
                &self.noises[self.consumed_noises..],
                &self.ability_inputs[self.consumed_ability_inputs..],
                &self.population_inputs[self.consumed_population_inputs..],
            );
            self.simulation.transport_planet_after_step();
            self.controls
                .observe_lifecycle(&self.simulation.actors, &self.simulation.vitality);
            self.consumed_population_inputs = self.population_inputs.len();
            if self
                .selection
                .as_ref()
                .is_some_and(|s| self.simulation.position(&s.id).is_none())
            {
                self.selection = None;
                self.view_revision += 1;
            }
            self.consumed_ability_inputs = self.ability_inputs.len();
            self.consumed_noises = self.noises.len();
            self.admission.ability_counts.fill(0);
            if self.recording() {
                self.inputs.push(self.input);
            } else {
                for noise in self.noises.drain(..) {
                    if self.recent_manual_noises.len() == RECENT_MANUAL_NOISES {
                        self.recent_manual_noises.pop_front();
                        self.dropped_manual_noises = self.dropped_manual_noises.saturating_add(1);
                    }
                    self.recent_manual_noises.push_back(noise);
                }
                while self
                    .recent_manual_noises
                    .front()
                    .is_some_and(|n| self.simulation.tick.saturating_sub(n.tick) >= 90)
                {
                    self.recent_manual_noises.pop_front();
                }
                self.consumed_noises = 0;
                self.ability_inputs.clear();
                self.consumed_ability_inputs = 0;
                self.population_inputs.clear();
                self.consumed_population_inputs = 0;
            }
        }
        self.prepare_planet_view()?;
        Ok(())
    }
    pub fn selected_mesh_components(&self) -> Option<&crate::mesh_topology::Selection> {
        self.mesh_selection.as_ref()
    }
    pub fn selected_id(&self) -> Option<&str> {
        self.selection
            .as_ref()
            .filter(|s| self.simulation.position(&s.id).is_some())
            .map(|s| s.id.as_str())
    }
    fn set_selection(&mut self, selection: Option<Selected>) {
        if self.selected_id() != selection.as_ref().map(|s| s.id.as_str()) {
            self.selection = selection;
            self.view_revision += 1;
        }
    }
    pub(crate) fn inherit_view(&mut self, source: &Engine) {
        self.debug_view = source.debug_view;
        self.selection = source.selected_id().and_then(|id| {
            self.simulation
                .definitions()
                .entities
                .iter()
                .position(|e| e.id == id)
                .map(|index| Selected {
                    id: id.into(),
                    index: index as u16,
                })
        });
    }
    fn inspect_selection(&self) -> Value {
        let selected=self.selection.as_ref().filter(|s|self.simulation.position(&s.id).is_some()).map(|s| {
            let e=&self.simulation.definitions().entities[s.index as usize];
            let trigger=e.trigger.as_ref().map(|config| {
                let state=self.simulation.triggers.volume_state(&e.id).unwrap();
                let occupants:Vec<_>=config.targets.iter().filter(|id|self.simulation.triggers.occupied(&e.id,id)==Some(true)).collect();
                json!({"occupants":occupants,"entered":state.entered,"exited":state.exited,"last_change_tick":state.last_change_tick})
            });
            json!({"id":e.id,"entity":e,"world_position":self.simulation.position(&e.id).expect("present selection"),
                "vitality":self.simulation.vitality.state.actors.get(self.simulation.actors.slot(s.index as usize) as usize).and_then(|h|*h).map(|h|json!({"units":h.units,"maximum":h.maximum,"scale":h.scale,"life":if h.units==0 {"dead"} else {"alive"}})),
                "npc":self.simulation.npcs.get(&e.id).map(|state|self.simulation.actors.inspect_state(self.simulation.definitions(),state)),"navigation":self.simulation.navigation.get(&e.id),
                "footsteps":self.simulation.footsteps.states.get(&e.id),"patrol":self.simulation.patrols.get(&e.id),"trigger":trigger})
        });
        json!({"tick":self.simulation.tick,"selection":selected})
    }
    pub fn render_frame(&self) -> crate::render::RenderFrame {
        let mut frame = crate::render::RenderFrame {
            clip: None,
            planet_air: None,
            suppress_diagnostic_grid: self.planet_active(),
            meshes: Default::default(),
            particles: Default::default(),
            distant_meshes: self.scene.distant_meshes.clone(),
            mesh_lods: self.mesh_lods.clone(),
            mesh_lod_settings: self.mesh_lod_settings,
            camera: self.scene.camera.clone(),
            light: self.scene.light.clone(),
            boxes: Vec::with_capacity(self.scene.entities.len()),
            vision_guides: Vec::new(),
            hearing_guides: Vec::new(),
            noise_guides: Vec::new(),
            route_guides: Vec::new(),
            truncated_navigation_paths: 0,
            selection: None,
            selection_mesh: None,
        };
        self.update_render_frame(&mut frame);
        frame
    }
    pub fn update_render_frame(&self, frame: &mut crate::render::RenderFrame) {
        self.update_render_frame_with_meshes(frame, &self.scene.meshes);
    }
    pub(crate) fn update_render_frame_with_meshes(
        &self,
        frame: &mut crate::render::RenderFrame,
        meshes: &crate::meshes::Document,
    ) {
        if self.planet_active() {
            frame.particles.clear();
        } else {
            frame.particles.update(
                &self.scene.particles,
                self.particle_preview.unwrap_or(self.simulation.tick),
            );
        }
        frame.clip = None;
        frame.planet_air = None;
        frame.suppress_diagnostic_grid = self.planet_active();
        if let Some(view) = &self.planet_view {
            frame.light = self.scene.light.clone();
            frame.light.direction = self
                .simulation
                .planet
                .as_ref()
                .expect("planet view source")
                .local_light();
            view.apply(frame);
            return;
        }
        let meshes = self
            .simulation
            .planet
            .as_ref()
            .map_or(meshes, |w| &w.meshes);
        let scene = self.simulation.definitions();
        frame.meshes.update_aerodynamic(
            meshes,
            &self.simulation,
            self.scene
                .plants
                .connections
                .iter()
                .map(|c| c.child_instance.as_str()),
        );
        if let Some(world) = &self.simulation.planet {
            frame
                .meshes
                .apply_smooth_normals(&world.normals)
                .expect("validated immutable planet normals");
        }
        let resolved_view = self.resolved_player_view();
        let view_actor = resolved_view
            .as_ref()
            .filter(|v| v.hide_body)
            .and_then(|_| self.player_view_actor());
        let mut avatar_poses = self.simulation.presentation_poses();
        if let Some(actor) = view_actor {
            let reference = self.simulation.actors.reference(actor as u16);
            avatar_poses.retain(|p| p.actor != reference.instance);
        }
        frame
            .meshes
            .append_avatars(meshes, &avatar_poses, |id| {
                self.simulation
                    .actors
                    .slot_for_instance(id)
                    .and_then(|slot| {
                        let e = &scene.entities[self.simulation.actors.entity(slot)];
                        self.simulation.position(&e.id).map(|p| (p, e.color))
                    })
            })
            .expect("validated avatar projection");
        frame.distant_meshes = self.scene.distant_meshes.clone();
        if let Some(world) = &self.simulation.planet {
            frame.distant_meshes = world.packs.clone();
        }
        frame.mesh_lods = self.mesh_lods.clone();
        frame.mesh_lod_settings = self.mesh_lod_settings;
        frame.camera = resolved_view
            .map(|v| v.camera)
            .unwrap_or_else(|| self.scene.camera.clone());
        frame.light = self.scene.light.clone();
        if let Some(world) = &self.simulation.planet {
            frame.light.direction = world.local_light();
        }
        frame.boxes.clear();
        frame.boxes.extend(
            scene
                .entities
                .iter()
                .enumerate()
                .filter(|(index, e)| {
                    e.visible != Some(false)
                        && view_actor
                            .is_none_or(|a| self.simulation.actors.entity(a as u16) != *index)
                        && self.simulation.position(&e.id).is_some()
                        && !avatar_poses.iter().any(|p| {
                            p.hide_body
                                && frame.meshes.drawn_avatars.contains(&p.actor)
                                && self
                                    .simulation
                                    .actors
                                    .entity_reference(*index)
                                    .is_some_and(|r| r.instance == p.actor)
                        })
                })
                .map(|(index, e)| crate::render::RenderBox {
                    geometry: e.geometry,
                    position: self.simulation.position(&e.id).expect("present placement"),
                    size: e.transform.size,
                    color: if self
                        .simulation
                        .vitality
                        .dead(self.simulation.actors.slot(index) as usize)
                    {
                        [0.22, 0.22, 0.24]
                    } else {
                        e.color
                    },
                }),
        );
        for shot in &self.simulation.abilities.state.projectiles.active {
            frame.boxes.push(crate::render::RenderBox {
                geometry: Default::default(),
                position: shot.position,
                size: [shot.half_extent * 2.; 3],
                color: [1., 0.72, 0.12],
            });
        }
        if self.debug_view.npc_routes {
            let periodic_mask = self.simulation.abilities.state.periodic.active.iter().fold(
                crate::actor_set::ActorSet::EMPTY,
                |mut mask, e| {
                    mask.insert(e.cast.target.unwrap() as usize);
                    mask
                },
            );
            for (a, &index) in self.simulation.actors.actors.iter().enumerate() {
                let e = &scene.entities[index as usize];
                if e.visible == Some(false) || (e.abilities.is_none() && e.vitality.is_none()) {
                    continue;
                }
                let Some(p) = self.simulation.position(&e.id) else {
                    continue;
                };
                let mut row = 0;
                if periodic_mask.contains(a) {
                    frame.boxes.push(crate::render::RenderBox {
                        geometry: Default::default(),
                        position: [p[0], p[1] + e.transform.size[1] * 0.5 + 0.08, p[2]],
                        size: [0.7, 0.06, 0.12],
                        color: [0.8, 0.4, 1.],
                    });
                }
                if let Some(h) = self.simulation.vitality.state.actors[a] {
                    let ratio = h.units as f32 / h.maximum as f32;
                    let y = p[1] + e.transform.size[1] * 0.5 + 0.25;
                    frame.boxes.push(crate::render::RenderBox {
                        geometry: Default::default(),
                        position: [p[0], y, p[2]],
                        size: [1., 0.09, 0.09],
                        color: if h.units == 0 {
                            [0.3, 0.3, 0.3]
                        } else {
                            [0.6, 0.06, 0.06]
                        },
                    });
                    if ratio > 0. {
                        frame.boxes.push(crate::render::RenderBox {
                            geometry: Default::default(),
                            position: [p[0] - 0.5 + ratio * 0.5, y, p[2] + 0.04],
                            size: [ratio, 0.07, 0.09],
                            color: [0.25, 1., 0.35],
                        });
                    }
                    row += 1;
                }
                for r in 0..scene.ability_catalog.resources.len() {
                    if let Some((units, capacity)) = self.simulation.abilities.resource_level(a, r)
                    {
                        let ratio = if capacity == 0 {
                            0.
                        } else {
                            units as f32 / capacity as f32
                        };
                        let y = p[1] + e.transform.size[1] * 0.5 + 0.25 + row as f32 * 0.16;
                        row += 1;
                        frame.boxes.push(crate::render::RenderBox {
                            geometry: Default::default(),
                            position: [p[0], y, p[2]],
                            size: [1., 0.09, 0.09],
                            color: [0.15, 0.15, 0.18],
                        });
                        if ratio > 0. {
                            frame.boxes.push(crate::render::RenderBox {
                                geometry: Default::default(),
                                position: [p[0] - 0.5 + ratio * 0.5, y, p[2] + 0.04],
                                size: [ratio, 0.07, 0.09],
                                color: [[0.1, 0.7, 1.], [0.3, 1., 0.3], [1., 0.35, 0.7]][r % 3],
                            });
                        }
                    }
                }
                // Show the earliest live buff deadline; inspection lists every contribution.
                if let Some(buff) = self
                    .simulation
                    .abilities
                    .state
                    .buffs
                    .active
                    .iter()
                    .filter(|b| b.owner == index && self.simulation.tick < b.expires_at)
                    .min_by_key(|b| (b.expires_at, b.id))
                {
                    let ratio = (buff.expires_at - self.simulation.tick) as f32
                        / (buff.expires_at - buff.applied_tick) as f32;
                    let y = p[1] + e.transform.size[1] * 0.5 + 0.25 + row as f32 * 0.16;
                    row += 1;
                    frame.boxes.push(crate::render::RenderBox {
                        geometry: Default::default(),
                        position: [p[0], y, p[2]],
                        size: [1., 0.09, 0.09],
                        color: [0.18, 0.25, 0.1],
                    });
                    frame.boxes.push(crate::render::RenderBox {
                        geometry: Default::default(),
                        position: [p[0] - 0.5 + ratio * 0.5, y, p[2] + 0.04],
                        size: [ratio, 0.07, 0.09],
                        color: [0.6, 1., 0.15],
                    });
                }
                if let Some(c) = self.simulation.abilities.state.casts[a] {
                    if c.aim_observation.is_some()
                        && let Some(crate::abilities::Aim::Point { position }) = c.aim
                    {
                        for axis in 0..3 {
                            let mut size = [0.04; 3];
                            size[axis] = 0.6;
                            frame.boxes.push(crate::render::RenderBox {
                                geometry: Default::default(),
                                position,
                                size,
                                color: [1., 0.72, 0.12],
                            });
                        }
                    }

                    let (ratio, color) = if let Some(progress) = c.channel {
                        let duration = progress.finishes_at - progress.started;
                        let remaining = progress.finishes_at.saturating_sub(self.simulation.tick);
                        (remaining as f32 / duration as f32, [0.1, 0.95, 0.85])
                    } else {
                        (
                            (self.simulation.tick.saturating_sub(c.started) as f32
                                / (c.execute_at - c.started).max(1) as f32)
                                .min(1.),
                            [1., 0.85, 0.1],
                        )
                    };
                    let y = p[1] + e.transform.size[1] * 0.5 + 0.25 + row as f32 * 0.16;
                    frame.boxes.push(crate::render::RenderBox {
                        geometry: Default::default(),
                        position: [p[0], y, p[2]],
                        size: [1., 0.09, 0.09],
                        color: [0.3, 0.25, 0.1],
                    });
                    if ratio > 0. {
                        frame.boxes.push(crate::render::RenderBox {
                            geometry: Default::default(),
                            position: [p[0] - 0.5 + ratio * 0.5, y, p[2] + 0.04],
                            size: [ratio, 0.07, 0.09],
                            color,
                        });
                    }
                }
            }
        }
        frame.selection_mesh = None;
        frame.selection = self
            .selection
            .as_ref()
            .filter(|s| self.simulation.position(&s.id).is_some())
            .map(|s| {
                let e = &scene.entities[s.index as usize];
                crate::render::RenderBox {
                    geometry: e.geometry,
                    position: self.simulation.position(&e.id).expect("present placement"),
                    size: e.transform.size,
                    color: [1., 0.8, 0.15],
                }
            });
        frame.vision_guides.clear();
        if self.debug_view.npc_vision {
            for entity in &scene.entities {
                let Some(config) = &entity.npc else { continue };
                if self.simulation.position(&entity.id).is_none() {
                    continue;
                }
                if self.simulation.npcs[&entity.id].phase == crate::perception::Phase::Dead {
                    continue;
                }
                let state = &self.simulation.npcs[&entity.id];
                let position = self
                    .simulation
                    .position(&entity.id)
                    .expect("present placement");
                let surface = scene
                    .entities
                    .iter()
                    .find(|e| e.id == config.surface)
                    .unwrap();
                let (range, fov, directional) = match config.awareness.as_ref().unwrap() {
                    Awareness::Perceptive {
                        vision_range,
                        fov_degrees,
                        ..
                    } => (*vision_range, *fov_degrees, true),
                    Awareness::Omniscient => (0., 0., false),
                };
                let color = if !config.enabled {
                    [0.4, 0.4, 0.4]
                } else if !directional {
                    [1., 0.15, 0.85]
                } else if state.observed_this_tick {
                    [0.1, 1., 0.2]
                } else if config.recognition.is_some()
                    && state.memory.is_none()
                    && state.knowledge.suspicious()
                {
                    [1., 1., 0.1]
                } else if state.memory.is_some() {
                    [1., 0.65, 0.05]
                } else {
                    [0.1, 0.65, 1.]
                };
                frame.vision_guides.push(crate::render::VisionGuide {
                    position,
                    top_y: position[1] + entity.transform.size[1] * 0.5 + 0.06,
                    ground_y: surface.transform.position[1]
                        + surface.transform.size[1] * 0.5
                        + 0.04,
                    facing_degrees: state.facing_degrees,
                    spatial_eye: state.facing_pitch_degrees.map(|_| {
                        [
                            position[0],
                            position[1] + entity.transform.size[1] * 0.35,
                            position[2],
                        ]
                    }),
                    pitch_degrees: state.facing_pitch_degrees.unwrap_or(0.),
                    range: if config.enabled { range } else { 0. },
                    fov_degrees: fov,
                    directional,
                    color,
                });
            }
        }
        frame.hearing_guides.clear();
        if self.debug_view.npc_hearing {
            for entity in &scene.entities {
                let Some(config) = &entity.npc else { continue };
                if self.simulation.position(&entity.id).is_none() {
                    continue;
                }
                if self.simulation.npcs[&entity.id].phase == crate::perception::Phase::Dead {
                    continue;
                }
                let Some(Awareness::Perceptive {
                    hearing: Some(hearing),
                    ..
                }) = &config.awareness
                else {
                    continue;
                };
                if !config.enabled {
                    continue;
                }
                let position = self
                    .simulation
                    .position(&entity.id)
                    .expect("present placement");
                let floor = scene
                    .entities
                    .iter()
                    .find(|e| e.id == config.surface)
                    .unwrap();
                frame.hearing_guides.push(crate::render::RingGuide {
                    center: [
                        position[0],
                        floor.transform.position[1] + floor.transform.size[1] * 0.5 + 0.05,
                        position[2],
                    ],
                    radius: hearing.range,
                    color: [0.75, 0.45, 1.],
                    emitter: false,
                });
            }
        }
        crate::debug_routes::fill(self, frame);
        if self.debug_view.npc_routes {
            for i in &self.simulation.abilities.state.persistent_areas.active {
                crate::area::sphere(
                    &mut frame.route_guides,
                    i.center,
                    i.area.radius,
                    if i.schedule().anchor == crate::persistent_area::Anchor::Source {
                        [0.35, 0.65, 1.]
                    } else {
                        [0.65, 0.9, 0.25]
                    },
                );
            }
            for r in self
                .simulation
                .abilities
                .state
                .areas
                .receipts
                .iter()
                .rev()
                .filter(|r| r.pulse.is_none())
                .take(8)
            {
                if self.simulation.tick.saturating_sub(r.tick) <= 60 {
                    crate::area::sphere(
                        &mut frame.route_guides,
                        r.center,
                        r.area.radius,
                        [1., 0.5, 0.1],
                    );
                }
            }
            for c in self.simulation.abilities.state.casts.iter().flatten() {
                let d = &scene.ability_catalog.abilities[c.definition as usize];
                if let Some(area) = d.area
                    && d.delivery.is_direct()
                {
                    let center = match c.aim {
                        Some(crate::abilities::Aim::Point { position }) => Some(position),
                        _ => c.target.and_then(|t| {
                            self.simulation
                                .position(self.simulation.actors.id(scene, t))
                        }),
                    };
                    if let Some(center) = center {
                        crate::area::sphere(
                            &mut frame.route_guides,
                            center,
                            area.radius,
                            [0.2, 0.9, 1.],
                        );
                    }
                }
            }
        }
        frame.noise_guides.clear();
        if self.debug_view.noise_events {
            let mut manual = self
                .recent_manual_noises
                .iter()
                .chain(self.noises.iter())
                .rev()
                .peekable();
            // Retained ability events and footsteps are VecDeque; index without allocating.
            let mut foot = self.simulation.footsteps.recent.len();
            let mut ability = self.simulation.abilities.state.recent_noises.len();
            for _ in 0..MAX_NOISES_PER_TICK {
                let m = manual.peek().copied();
                let f = foot
                    .checked_sub(1)
                    .map(|i| &self.simulation.footsteps.recent[i]);
                let a = ability
                    .checked_sub(1)
                    .map(|i| &self.simulation.abilities.state.recent_noises[i]);
                let next = [m, f, a]
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, n)| n.map(|n| (i, n)))
                    .max_by_key(|(i, n)| (n.tick, std::cmp::Reverse(*i)));
                let noise = next.map(|(i, n)| {
                    match i {
                        0 => {
                            manual.next();
                        }
                        1 => foot -= 1,
                        _ => ability -= 1,
                    };
                    n
                });
                let Some(noise) = noise else { break };
                if self.simulation.tick.saturating_sub(noise.tick) >= 90 {
                    break;
                }
                frame.noise_guides.push(crate::render::RingGuide {
                    center: noise.position,
                    radius: noise.radius,
                    color: [1., 0.55, 0.1],
                    emitter: true,
                });
            }
        }
    }
    fn npc_snapshot(&self) -> Value {
        Value::Object(
            self.simulation
                .npcs
                .iter()
                .map(|(id, state)| {
                    (
                        id.clone(),
                        self.simulation
                            .actors
                            .inspect_state(self.simulation.definitions(), state),
                    )
                })
                .collect(),
        )
    }
    fn inspect_targeting(&self) -> Value {
        let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)|{
            let entity=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap();let config=entity.npc.as_ref().unwrap();
            let decoded=self.simulation.actors.inspect_state(self.simulation.definitions(),state);
            (id.clone(),json!({"explicit_target":config.target,"policy":config.targeting,"faction":entity.faction,"decision_tick":state.decision_tick,"selection":decoded["targeting"],"memory":decoded["memory"],"recognition":config.recognition,"knowledge":decoded["knowledge"]}))
        }).collect();
        json!({"tick":self.simulation.tick,"npcs":npcs})
    }
    fn inspect_knowledge(&self) -> Value {
        let scene = self.simulation.definitions();
        let npcs: serde_json::Map<_, _> = self
            .simulation
            .npcs
            .iter()
            .map(|(id, state)| {
                let config = scene
                    .entities
                    .iter()
                    .find(|e| e.id == *id)
                    .unwrap()
                    .npc
                    .as_ref()
                    .unwrap()
                    .recognition;
                let tracks = self
                    .simulation
                    .actors
                    .inspect_knowledge(self.simulation.definitions(), state);
                (
                    id.clone(),
                    json!({"config":config,"decision_tick":state.decision_tick,"tracks":tracks}),
                )
            })
            .collect();
        json!({"tick":self.simulation.tick,"npcs":npcs})
    }
    pub fn snapshot(&self) -> Value {
        json!({"execution":self.inspect_execution(),"revision":self.revision,"selection":self.selected_id(),"mesh_selection":self.mesh_selection,"debug_view":self.debug_view,"view_revision":self.view_revision,"tick":self.simulation.tick,"fixed_dt":FIXED_DT,"playing":self.playing,"input":self.input,"controls":self.inspect_controls(),"player_motion":(self.simulation.player_count()<=1).then(||self.simulation.movement_profile(self.input.mode)),"scene":self.scene,"runtime_wind":self.simulation.wind_primaries(),"runtime_positions":self.simulation.positions(),"contacts":self.simulation.contacts(),"navigation":self.simulation.navigation,"passages":self.simulation.passages,"mobility":self.simulation.mobility.states,"character_controllers":self.simulation.character_controllers,"npcs":self.npc_snapshot(),"history":{"undo":self.undo.len(),"redo":self.redo.len()},"recorded_ticks":self.inputs.len(),"footsteps":self.simulation.footsteps.states,"patrols":self.simulation.patrols,"triggers":self.simulation.triggers.inspect(),"generated_footsteps":self.simulation.footsteps.total_emitted,"recorded_noises":self.noises.len(),"pending_noises":self.noises.len()-self.consumed_noises,"abilities":self.inspect_abilities(),"vitality":self.simulation.vitality.inspect(self.simulation.definitions(),&self.simulation.actors),"population":self.inspect_population(),"generators":self.inspect_generators(),"interactions":self.simulation.interactions.state,"presentation":self.simulation.presentation,"build_id":env!("IGNIS_BUILD_ID")})
    }
    pub fn replay(&self) -> Replay {
        Replay {
            baseline: self.baseline.clone(),
            expected_presentation: self.simulation.presentation.clone(),
            execution_mode: self.execution_mode,
            version: REPLAY_VERSION,
            engine_version: env!("CARGO_PKG_VERSION").into(),
            physics_version: "rapier3d-0.30.1".into(),
            build_id: env!("IGNIS_BUILD_ID").into(),
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            fixed_dt: FIXED_DT,
            scene: self.scene.clone(),
            inputs: self.inputs.clone(),
            control_events: self.control_events.clone(),
            expected_controls: self.controls.state.clone(),
            noises: self.noises.clone(),
            ability_inputs: self.ability_inputs.clone(),
            population_inputs: self.population_inputs.clone(),
            expected_population: self.simulation.population.clone(),
            expected_interactions: self.simulation.interactions.state.clone(),
            expected_generators: self.simulation.generators.clone(),
            expected_wind: self.simulation.wind_primaries(),
            expected_meshes: self.scene.meshes.poses(self.simulation.tick),
            expected_instances: self.simulation.actors.instance_snapshot(),
            expected_abilities: self.simulation.abilities.state.clone(),
            expected_vitality: self.simulation.vitality.state.clone(),
            expected_navigation: self.simulation.navigation.clone(),
            expected_character_controllers: self.simulation.character_controllers.clone(),
            expected_mobility: self.simulation.mobility.states.clone(),
            expected_passages: self.simulation.passages.clone(),
            expected_positions: self.simulation.positions(),
            expected_npcs: self.simulation.npcs.clone(),
            expected_footsteps: self.simulation.footsteps.clone(),
            expected_triggers: self.simulation.triggers.state.clone(),
            expected_patrols: self.simulation.patrols.clone(),
        }
    }
    /// Reconstruct a prefix without changing any live engine. Events stamped at the
    /// selected tick are present but pending; the next step consumes them.
    pub fn replay_at(replay: &Replay, tick: Option<u64>) -> Result<Self> {
        if replay.execution_mode != ExecutionMode::Recorded {
            return Err(invalid("Replay requires recorded execution"));
        }
        if replay.build_id != env!("IGNIS_BUILD_ID")
            || replay.version != REPLAY_VERSION
            || replay.engine_version != env!("CARGO_PKG_VERSION")
            || replay.physics_version != "rapier3d-0.30.1"
            || replay.fixed_dt != FIXED_DT
        {
            return Err(invalid("Replay version/runtime mismatch"));
        }
        if replay.inputs.len() > MAX_TICKS {
            return Err(invalid("Replay exceeds 36000 ticks"));
        }
        let tick = tick.unwrap_or(replay.inputs.len() as u64);
        if tick > replay.inputs.len() as u64 {
            return Err(invalid("Replay tick exceeds recorded duration"));
        }
        for input in &replay.inputs {
            Self::validate_input(*input)?;
        }
        if replay.noises.len() > MAX_NOISES {
            return Err(invalid("Replay exceeds 4096 noises"));
        }
        let mut count = 0;
        let mut engine = if let Some(bytes) = &replay.baseline {
            if replay.version < 107 {
                return Err(invalid("Checkpoint baseline requires replay107"));
            }
            let candidate = Self::from_checkpoint(bytes, Some(ExecutionMode::Recorded))?;
            let mut actual_scene = candidate.scene.clone();
            let mut recorded_scene = replay.scene.clone();
            for scene in [&mut actual_scene, &mut recorded_scene] {
                if let Some(r) = &mut scene.navigation_generation {
                    r.cached = None;
                }
            }
            if actual_scene != recorded_scene {
                return Err(invalid("Replay scene differs from checkpoint baseline"));
            }
            candidate
        } else {
            Self::new(replay.scene.clone())?
        };
        let start_tick = engine.baseline_tick;
        let end_tick = start_tick
            .checked_add(replay.inputs.len() as u64)
            .filter(|t| *t <= MAX_RUNTIME_TICK)
            .ok_or_else(|| invalid("Replay absolute tick exhausted"))?;
        let selected_absolute = start_tick + tick;
        let sequences = engine.baseline_sequences;
        let mut last_tick = None;
        for (index, noise) in replay.noises.iter().enumerate() {
            noise.validate()?;
            if noise.id
                != crate::noise::event_id(
                    crate::noise::Domain::Manual,
                    sequences[0]
                        .checked_add(index as u64)
                        .filter(|s| *s <= crate::noise::MAX_SERIAL)
                        .ok_or_else(|| invalid("Noise sequence exhausted"))?,
                )
                || noise.emitter_index.is_some()
                || noise.emitter_instance.is_some()
                || noise.tick < start_tick
                || noise.tick > end_tick
                || noise.tick.saturating_sub(start_tick) >= MAX_TICKS as u64
                || last_tick.is_some_and(|t| noise.tick < t)
            {
                return Err(invalid("Invalid noise order, id or tick"));
            }
            count = if last_tick == Some(noise.tick) {
                count + 1
            } else {
                1
            };
            if count > MAX_NOISES_PER_TICK {
                return Err(invalid("Too many noises in one tick"));
            }
            last_tick = Some(noise.tick);
        }

        if replay.ability_inputs.len() > crate::abilities::MAX_INPUTS {
            return Err(invalid("Too many ability inputs"));
        }
        let mut last = None;
        let mut count = 0;
        let mut by_actor = vec![0u8; engine.simulation.actors.actors.len()];
        for (index, e) in replay.ability_inputs.iter().enumerate() {
            if e.sequence
                != sequences[1]
                    .checked_add(index as u64)
                    .ok_or_else(|| invalid("Ability sequence exhausted"))?
                || e.tick < start_tick
                || e.tick > end_tick
                || e.tick.saturating_sub(start_tick) >= MAX_TICKS as u64
                || last.is_some_and(|t| e.tick < t)
            {
                return Err(invalid("Invalid ability input order, sequence or tick"));
            }
            count = if last == Some(e.tick) { count + 1 } else { 1 };
            if count > crate::abilities::input_capacity(by_actor.len()) {
                return Err(invalid("Too many ability inputs in tick"));
            }
            if last != Some(e.tick) {
                by_actor.fill(0);
            }
            let actor_count = by_actor
                .get_mut(e.actor as usize)
                .ok_or_else(|| invalid("Invalid ability actor slot"))?;
            if *actor_count as usize >= crate::abilities::PER_ACTOR_PER_TICK {
                return Err(invalid("Too many ability inputs for one actor in tick"));
            }
            *actor_count += 1;
            last = Some(e.tick);
        }
        let mut population_order = replay.population_inputs.clone();
        for e in &mut population_order {
            e.tick = e
                .tick
                .checked_sub(start_tick)
                .ok_or_else(|| invalid("Population before baseline"))?;
            e.sequence = e
                .sequence
                .checked_sub(sequences[2])
                .ok_or_else(|| invalid("Population sequence before baseline"))?;
        }
        crate::population::validate_order(&population_order, replay.inputs.len())?;
        let mut control_order = replay.control_events.clone();
        for e in &mut control_order {
            e.tick = e
                .tick
                .checked_sub(start_tick)
                .ok_or_else(|| invalid("Control before baseline"))?;
        }
        crate::controls::validate_order(&control_order, replay.inputs.len())?;
        let mut control_cursor = 0;
        let mut population_cursor = 0;
        let mut cursor = 0;
        let mut ability_cursor = 0;
        for input in &replay.inputs[..tick as usize] {
            while control_cursor < replay.control_events.len()
                && replay.control_events[control_cursor].tick == engine.simulation.tick
            {
                engine.admit_control(replay.control_events[control_cursor].operation)?;
                control_cursor += 1;
            }
            if *input != Input::default()
                && (engine.simulation.player_count() > 1
                    || !engine.controls.state.bindings.is_empty())
            {
                return Err(invalid("Replay contains ambiguous legacy input"));
            }
            let start = cursor;
            while cursor < replay.noises.len()
                && replay.noises[cursor].tick == engine.simulation.tick
            {
                cursor += 1;
            }
            let ability_start = ability_cursor;
            while ability_cursor < replay.ability_inputs.len()
                && replay.ability_inputs[ability_cursor].tick == engine.simulation.tick
            {
                engine.simulation.abilities.validate_entry(
                    replay.ability_inputs[ability_cursor],
                    &engine.simulation.actors,
                )?;
                ability_cursor += 1;
            }
            let population_start = population_cursor;
            while population_cursor < replay.population_inputs.len()
                && replay.population_inputs[population_cursor].tick == engine.simulation.tick
            {
                engine
                    .simulation
                    .validate_population(replay.population_inputs[population_cursor])?;
                population_cursor += 1;
            }
            let controls = engine
                .controls
                .inputs(&engine.simulation.actors, &engine.simulation.vitality);
            engine.simulation.step_with_controls(
                *input,
                controls,
                &replay.noises[start..cursor],
                &replay.ability_inputs[ability_start..ability_cursor],
                &replay.population_inputs[population_start..population_cursor],
            );
            engine
                .controls
                .observe_lifecycle(&engine.simulation.actors, &engine.simulation.vitality);
        }
        engine.input = replay
            .inputs
            .get(tick.wrapping_sub(1) as usize)
            .copied()
            .unwrap_or_default();
        while control_cursor < replay.control_events.len()
            && replay.control_events[control_cursor].tick == selected_absolute
        {
            engine.admit_control(replay.control_events[control_cursor].operation)?;
            control_cursor += 1;
        }

        engine.consumed_population_inputs = population_cursor;
        while population_cursor < replay.population_inputs.len()
            && replay.population_inputs[population_cursor].tick == selected_absolute
        {
            engine
                .simulation
                .validate_population(replay.population_inputs[population_cursor])?;
            population_cursor += 1;
        }
        engine
            .population_inputs
            .extend_from_slice(&replay.population_inputs[..population_cursor]);
        engine.consumed_ability_inputs = ability_cursor;
        while ability_cursor < replay.ability_inputs.len()
            && replay.ability_inputs[ability_cursor].tick == selected_absolute
        {
            engine.simulation.abilities.validate_entry(
                replay.ability_inputs[ability_cursor],
                &engine.simulation.actors,
            )?;
            ability_cursor += 1;
        }
        engine
            .ability_inputs
            .extend_from_slice(&replay.ability_inputs[..ability_cursor]);
        engine.consumed_noises = cursor;
        while cursor < replay.noises.len() && replay.noises[cursor].tick == selected_absolute {
            cursor += 1;
        }
        engine.noises.extend_from_slice(&replay.noises[..cursor]);
        engine
            .inputs
            .extend_from_slice(&replay.inputs[..tick as usize]);
        engine.admission.ability = sequences[1] + engine.ability_inputs.len() as u64;
        engine.admission.population = sequences[2] + engine.population_inputs.len() as u64;
        engine.admission.noise = sequences[0] + engine.noises.len() as u64;
        for entry in &engine.ability_inputs[engine.consumed_ability_inputs..] {
            engine.admission.ability_counts[entry.actor as usize] += 1;
        }
        Ok(engine)
    }
    /// Verification uses the same reconstruction as inspection and capture.
    pub fn verify_replay(replay: &Replay) -> Result<Value> {
        Ok(Self::replay_at(replay, None)?.compare_replay(replay))
    }
    pub(crate) fn compare_replay(&self, replay: &Replay) -> Value {
        let simulation = &self.simulation;
        let actual = simulation.positions();
        let generators_matched = simulation.generators == replay.expected_generators;
        let population_matched = simulation.population == replay.expected_population;
        let wind_matched = simulation.wind_primaries() == replay.expected_wind;
        let meshes_matched = self.scene.meshes.poses(simulation.tick) == replay.expected_meshes;
        let instances_matched = simulation.actors.instance_snapshot() == replay.expected_instances;
        let controls_matched = self.controls.state == replay.expected_controls;
        let interactions_matched = simulation.interactions.state == replay.expected_interactions;
        json!({"wind_matched":wind_matched,"character_controllers_matched":simulation.character_controllers==replay.expected_character_controllers,"controls_matched":controls_matched,"meshes_matched":meshes_matched,"generators_matched":generators_matched,"population_matched":population_matched,"instances_matched":instances_matched,"vitality_matched":simulation.vitality.state==replay.expected_vitality,"abilities_matched":simulation.abilities.state==replay.expected_abilities,"interactions_matched":interactions_matched,"presentation_matched":simulation.presentation==replay.expected_presentation,"matched":wind_matched && simulation.presentation==replay.expected_presentation && interactions_matched && simulation.character_controllers==replay.expected_character_controllers && controls_matched && meshes_matched && generators_matched && population_matched && instances_matched && simulation.vitality.state==replay.expected_vitality && simulation.abilities.state==replay.expected_abilities && actual==replay.expected_positions && simulation.mobility.states==replay.expected_mobility && simulation.passages==replay.expected_passages && simulation.navigation==replay.expected_navigation && simulation.npcs==replay.expected_npcs && simulation.footsteps==replay.expected_footsteps && simulation.patrols==replay.expected_patrols && simulation.triggers.state==replay.expected_triggers,"mobility_matched":simulation.mobility.states==replay.expected_mobility,"passages_matched":simulation.passages==replay.expected_passages,"triggers_matched":simulation.triggers.state==replay.expected_triggers,"patrols_matched":simulation.patrols==replay.expected_patrols,"footsteps_matched":simulation.footsteps==replay.expected_footsteps,"npcs_matched":simulation.npcs==replay.expected_npcs,"navigation_matched":simulation.navigation==replay.expected_navigation,"ticks":simulation.tick,"actual_positions":actual,"expected_positions":replay.expected_positions,"recorded_platform":replay.platform,"current_platform":format!("{}-{}",std::env::consts::OS,std::env::consts::ARCH)})
    }
    fn inspect_instances(&self) -> Value {
        let entries: Vec<_> = self.simulation.definitions().entities.iter().enumerate().filter_map(|(i,e)| {
            let r=self.simulation.actors.entity_reference(i)?;
            let actor=self.simulation.actors.slot(i) as usize;
            Some(json!({"authored_id":e.id,"instance_id":format!("{:016x}",r.instance),"generation":format!("{:016x}",r.generation),
                "reference":format!("ignis:{}:{:016x}",self.execution_scope,r.instance),
                "life_index":self.simulation.vitality.state.lives.get(actor),"actor":actor<crate::targeting::MAX_ACTORS}))
        }).collect();
        json!({"execution_scope":self.execution_scope,"tick":self.simulation.tick,"instances":entries,"dynamic_population":true,"spawn_supported":true,"despawn_supported":true})
    }
    fn population_operation(&self, operation: crate::population::Operation) -> Value {
        match operation {
            crate::population::Operation::Interact {
                instance,
                life,
                request,
                ..
            } => {
                json!({"kind":"interact","instance_id":instance.id(),"life":life,"request":request})
            }
            crate::population::Operation::OrderNpc {
                instance,
                life,
                order,
                ..
            } => json!({"kind":"order_npc","instance_id":instance.id(),"life":life,"order":order}),
            crate::population::Operation::ControlGenerator { generator, action } => {
                json!({"kind":"control_generator","generator":self.scene.npc_generators[generator as usize].id,"action":action})
            }
            crate::population::Operation::Despawn { instance, .. } => {
                json!({"kind":"despawn","instance_id":instance.id(),"authored_id":self.simulation.actors.label(self.simulation.definitions(),instance)})
            }
            crate::population::Operation::Spawn {
                template,
                position,
                target,
            } => {
                json!({"kind":"spawn","template":self.scene.npc_templates[template as usize].id,"position":position,"target_instance":target.map(|r|r.id())})
            }
        }
    }
    fn inspect_generators(&self) -> Value {
        let (counts, waves): (Vec<_>, Vec<_>) = self.simulation.generators.generators.iter().enumerate().map(|(i,g)| {
            let mut present=0;let mut alive=0;let mut wave_present=0;let mut wave_alive=0;
            for m in g.members.iter().flatten().filter(|m|self.simulation.actors.matches(m.actor,m.instance)) {
                present+=1;
                let living=!self.simulation.vitality.dead(m.actor as usize);
                alive+=usize::from(living);
                if m.wave==g.wave.map(|w|w.current) {wave_present+=1;wave_alive+=usize::from(living);}
            }
            let config=&self.scene.npc_generators[i];
            let wave=g.wave.map(|w| {
                use crate::waves::Phase;
                let waiting_for=match w.phase {
                    Phase::Delay=>"delay",Phase::WaitingForClear=>"clear_condition",Phase::Completed=>"none",
                    Phase::Spawning if alive>=config.target_alive as usize=>"living_limit",
                    Phase::Spawning if present>=8=>"member_capacity",
                    Phase::Spawning if self.simulation.tick<g.next_attempt_tick=>"cadence",
                    Phase::Spawning=>"candidate_attempt",
                };
                json!({"index":w.current,"ordinal":w.current+1,"total":config.waves.len(),"phase":w.phase,
                    "births":w.births,"count":config.waves[w.current as usize].count,"present":wave_present,"alive":wave_alive,
                    "due_tick":w.due_tick,"completed_tick":w.completed_tick,"waiting_for":waiting_for})
            });
            (json!({"present":present,"alive":alive,"dead":present-alive}),wave)
        }).unzip();
        let controls: Vec<_> = self.simulation.generators.generators.iter().enumerate().map(|(i, g)| {
            use crate::generator_control::Status;
            let config = &self.scene.npc_generators[i];
            let clock = g.paused_since.unwrap_or(self.simulation.tick);
            let terminal = if g.status == Status::Cancelled { Some("cancelled") }
                else if g.finished(config.max_births) { Some(if g.wave.is_some() { "completed" } else { "budget_exhausted" }) }
                else { None };
            json!({"id":config.id,"status":g.status,"terminal":terminal,"production_enabled":g.production_enabled() && terminal.is_none(),"pending_template":g.pending_template.then(||self.scene.npc_templates[g.template as usize].id.as_str()),"template_draws":g.template_draws,"condition_open":g.activation.map(|a|a.open),"activation":g.activation,"paused_since":g.paused_since,"cancelled_tick":g.cancelled_tick,
                "remaining_delay_ticks":g.wave.filter(|w|w.phase==crate::waves::Phase::Delay).map(|w|w.due_tick.saturating_sub(clock)),
                "remaining_cadence_ticks":g.next_attempt_tick.saturating_sub(clock)})
        }).collect();
        json!({"tick":self.simulation.tick,"definitions":self.scene.npc_generators,"support_surface_ids":self.scene.entities.iter().enumerate().filter(|(_,e)|e.body==Some(BodyKind::Fixed)).map(|(i,e)|(i.to_string(),json!(e.id))).collect::<serde_json::Map<_,_>>(),"template_ids":self.scene.npc_templates.iter().map(|t|t.id.as_str()).collect::<Vec<_>>(),"controls":controls,"counts":counts,"waves":waves,"state":self.simulation.generators,"limits":{"generators":crate::generators::MAX_GENERATORS,"points":crate::generators::MAX_POINTS,"template_choices":crate::template_selection::MAX_CHOICES,"template_weight":crate::template_selection::MAX_WEIGHT,"waves_per_generator":crate::waves::MAX_WAVES,"wave_events":crate::waves::EVENTS,"receipts":crate::generators::RECEIPTS}})
    }
    fn inspect_population(&self) -> Value {
        let pending: Vec<_> = self.population_inputs[self.consumed_population_inputs..]
            .iter()
            .map(|e| {
                let mut v = self.population_operation(e.operation);
                v["operation"] = v["kind"].clone();
                v["tick"] = json!(e.tick);
                v["request_id"] = json!(e.sequence);
                v
            })
            .collect();
        let receipts: Vec<_> = self
            .simulation
            .population
            .receipts
            .iter()
            .map(|r| {
                let mut v = self.population_operation(r.operation);
                v["operation"] = v["kind"].clone();
                v["tick"] = json!(r.tick);
                v["request_id"] = json!(r.request_id);
                v["outcome"] = json!(r.outcome);
                if let Some(instance) = r.instance {
                    v["instance_id"] = json!(instance.id());
                    v["authored_id"] = json!(
                        self.simulation
                            .actors
                            .label(self.simulation.definitions(), instance)
                    );
                }
                v
            })
            .collect();
        let present = self
            .simulation
            .actors
            .actors
            .iter()
            .filter(|&&e| {
                self.simulation
                    .actors
                    .entity_reference(e as usize)
                    .is_some()
            })
            .count();
        json!({"tick":self.simulation.tick,"present_actors":present,"spawn_supported":true,"despawn_supported":true,"pending":pending,"receipts":receipts,"processed":self.simulation.population.processed,"dropped_receipts":self.simulation.population.dropped_receipts,"recorded_inputs":self.population_inputs.len(),"limits":{"npcs":crate::templates::capacity(&self.scene),"templates":crate::templates::MAX_TEMPLATES,"per_tick":crate::population::PER_TICK,"inputs":self.recording().then_some(crate::population::MAX_INPUTS),"receipts":crate::population::RECEIPTS}})
    }
    fn queue_population(&mut self, operation: crate::population::Operation) -> Result<Value> {
        if self.recording() && self.population_inputs.len() >= crate::population::MAX_INPUTS
            || self.population_inputs.len() - self.consumed_population_inputs
                >= crate::population::PER_TICK
            || !self.clock_open()
        {
            return Err(invalid("Population input budget exhausted"));
        }
        let entry = crate::population::Entry {
            tick: self.simulation.tick,
            sequence: self.admission.population,
            operation,
        };
        let next = self
            .admission
            .population
            .checked_add(1)
            .ok_or_else(|| invalid("Population sequence exhausted"))?;
        self.simulation.validate_population(entry)?;
        self.population_inputs.push(entry);
        self.admission.population = next;
        let mut result = self.population_operation(operation);
        result["accepted"] = json!(true);
        result["request_id"] = json!(entry.sequence);
        result["tick"] = json!(entry.tick);
        result["phase"] = json!("pending");
        Ok(result)
    }
    fn dialogue_id(&self, token: &str) -> Result<u64> {
        let (scope, id) = token
            .strip_prefix("ignis-dialogue:")
            .and_then(|s| s.split_once(':'))
            .ok_or_else(|| invalid("Malformed dialogue handle"))?;
        if scope != self.execution_scope
            || id.len() != 16
            || !id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid("Stale or malformed dialogue handle"));
        }
        u64::from_str_radix(id, 16).map_err(|_| invalid("Malformed dialogue handle"))
    }
    fn queue_interaction(
        &mut self,
        actor: u16,
        request: crate::interactions::Request,
    ) -> Result<Value> {
        self.queue_population(crate::population::Operation::Interact {
            actor,
            instance: self.simulation.actors.reference(actor),
            life: self.simulation.vitality.state.lives[actor as usize],
            request,
        })
    }
    fn queue_despawn(&mut self, id: &str) -> Result<Value> {
        let actor = self.actor_slot(id)?;
        self.queue_population(crate::population::Operation::Despawn {
            actor,
            instance: self.simulation.actors.reference(actor),
        })
    }
    fn queue_npc_orders(
        &mut self,
        mut members: Vec<u16>,
        order: crate::npc_cooperation::Order,
    ) -> Result<Value> {
        if members.is_empty() {
            return Err(invalid("No living NPC recipients"));
        }
        if !self.clock_open()
            || self.population_inputs.len() - self.consumed_population_inputs + members.len()
                > crate::population::PER_TICK
            || (self.recording()
                && self.population_inputs.len() + members.len() > crate::population::MAX_INPUTS)
        {
            return Err(invalid(
                "Population budget cannot admit the complete group order",
            ));
        }
        let next = self
            .admission
            .population
            .checked_add(members.len() as u64)
            .ok_or_else(|| invalid("Population sequence exhausted"))?;
        members.sort_by_key(|&a| {
            let e = &self.simulation.definitions().entities[self.simulation.actors.entity(a)];
            let role = e
                .npc
                .as_ref()
                .and_then(|n| n.cooperation)
                .map_or(0, |c| c.role);
            let profile = self
                .simulation
                .npcs
                .get(&e.id)
                .and_then(|s| s.mind_profile)
                .or_else(|| {
                    e.npc
                        .as_ref()
                        .and_then(|n| n.mind)
                        .map(|m| m.resolve(self.simulation.actors.reference(a).instance))
                });
            let risk = profile.map_or(0, |p| i32::from(p.traits[4]) - i32::from(p.traits[3]));
            (
                role,
                std::cmp::Reverse(risk),
                self.simulation.actors.reference(a).instance,
            )
        });
        let mut entries = Vec::with_capacity(members.len());
        for (ordinal, &actor) in members.iter().enumerate() {
            let instance = self.simulation.actors.reference(actor);
            let e = &self.simulation.definitions().entities[self.simulation.actors.entity(actor)];
            let configuration = e
                .npc
                .as_ref()
                .ok_or_else(|| invalid("Orders support NPCs only"))?
                .cooperation
                .unwrap_or_default();
            let frozen = order.freeze(configuration, ordinal, members.len())?;
            if let crate::npc_cooperation::Intent::Move { position, .. } = frozen.intent
                && e.npc
                    .as_ref()
                    .and_then(|n| n.territory)
                    .is_some_and(|t| !t.contains(position))
            {
                return Err(invalid(
                    "Order destination must be inside the recipient territory",
                ));
            }
            let entry = crate::population::Entry {
                tick: self.simulation.tick,
                sequence: self.admission.population + ordinal as u64,
                operation: crate::population::Operation::OrderNpc {
                    actor,
                    instance,
                    life: self.simulation.vitality.state.lives[actor as usize],
                    order: frozen,
                },
            };
            self.simulation.validate_population(entry)?;
            entries.push(entry);
        }
        let recipients:Vec<_>=entries.iter().map(|e|json!({"request_id":e.sequence,"operation":self.population_operation(e.operation)})).collect();
        self.population_inputs.extend(entries);
        self.admission.population = next;
        Ok(
            json!({"accepted":true,"tick":self.simulation.tick,"phase":"pending","recipients":recipients}),
        )
    }
    fn queue_spawn(
        &mut self,
        template: &str,
        position: [f32; 3],
        target: Option<&str>,
    ) -> Result<Value> {
        let t = self
            .scene
            .npc_templates
            .iter()
            .position(|e| e.id == template)
            .ok_or_else(|| invalid("Unknown NPC template"))?;
        let target = target
            .or(self.scene.npc_templates[t]
                .npc
                .as_ref()
                .unwrap()
                .target
                .as_deref())
            .map(|id| {
                self.actor_slot(id)
                    .map(|a| self.simulation.actors.reference(a))
            })
            .transpose()?;
        self.queue_population(crate::population::Operation::Spawn {
            template: t as u8,
            position,
            target,
        })
    }
    fn control_token(&self, control: u64) -> String {
        format!("ignis-control:{}:{control:016x}", self.execution_scope)
    }
    fn control_id(&self, token: &str) -> Result<u64> {
        let (scope, id) = token
            .strip_prefix("ignis-control:")
            .and_then(|s| s.split_once(':'))
            .ok_or_else(|| invalid("Malformed control handle"))?;
        if scope != self.execution_scope
            || id.len() != 16
            || !id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid("Stale or malformed control handle"));
        }
        let id = u64::from_str_radix(id, 16).map_err(|_| invalid("Malformed control handle"))?;
        self.controls.binding(id)?;
        Ok(id)
    }
    fn apply_control(&mut self, operation: crate::controls::Operation) -> Result<()> {
        self.controls.apply(
            operation,
            &self.simulation.actors,
            self.simulation.definitions(),
            &self.simulation.vitality,
        )?;
        if matches!(
            operation,
            crate::controls::Operation::Bind { .. } | crate::controls::Operation::Release { .. }
        ) {
            self.input = Input::default();
            self.input_revision += 1;
        }
        Ok(())
    }
    fn admit_control(
        &mut self,
        operation: crate::controls::Operation,
    ) -> Result<crate::controls::Entry> {
        let tick = self.simulation.tick;
        let count = if self.admission.control_tick == tick {
            self.admission.controls_at_tick
        } else {
            0
        };
        let touched_actor = match operation {
            crate::controls::Operation::Bind { actor, .. } => {
                self.simulation.actors.actor_slot(actor)
            }
            crate::controls::Operation::Input { control, .. }
            | crate::controls::Operation::Release { control } => self
                .simulation
                .actors
                .actor_slot(self.controls.binding(control)?.actor),
        };
        let coalesced = !self.recording()
            && touched_actor.is_some_and(|a| self.admission.control_touched[a] == tick);
        let budget_delta = usize::from(!coalesced);
        let reserved_releases = match operation {
            crate::controls::Operation::Bind { .. } => self.controls.state.bindings.len() + 1,
            crate::controls::Operation::Input { .. } => self.controls.state.bindings.len(),
            crate::controls::Operation::Release { .. } => {
                self.controls.state.bindings.len().saturating_sub(1)
            }
        };
        if self.recording()
            && self.control_events.len() + 1 + reserved_releases > crate::controls::MAX_EVENTS
            || count + budget_delta + reserved_releases > crate::controls::PER_TICK
            || !self.clock_open()
        {
            return Err(invalid("Control event budget exceeded"));
        }
        let sequence = self.admission.control;
        let next = sequence
            .checked_add(1)
            .ok_or_else(|| invalid("Control event sequence exhausted"))?;
        self.apply_control(operation)?;
        let entry = crate::controls::Entry {
            sequence,
            tick,
            operation,
        };
        if self.recording() {
            self.control_events.push(entry);
        }
        self.admission.control = next;
        self.admission.control_tick = tick;
        self.admission.controls_at_tick = count + budget_delta;
        if !self.recording()
            && let Some(actor) = touched_actor
        {
            self.admission.control_touched[actor] = tick;
        }
        Ok(entry)
    }
    fn record_control(&mut self, operation: crate::controls::Operation) -> Result<Value> {
        let entry = self.admit_control(operation)?;
        let tick = entry.tick;
        let sequence = entry.sequence;
        let control = match operation {
            crate::controls::Operation::Bind { control, .. }
            | crate::controls::Operation::Input { control, .. }
            | crate::controls::Operation::Release { control } => control,
        };
        Ok(
            json!({"control":self.control_token(control),"tick":tick,"sequence":sequence,"applied":true}),
        )
    }
    fn inspect_controls(&self) -> Value {
        let actors = &self.simulation.actors;
        let players: Vec<_> = actors.actors.iter().enumerate().filter_map(|(a,&e)| {
            let entity = &self.simulation.definitions().entities[e as usize];
            if entity.body != Some(BodyKind::Player) { return None; }
            let reference = actors.entity_reference(e as usize)?;
            let binding = self.controls.state.bindings.values().find(|b| b.actor == reference);
            let input = if self.simulation.vitality.dead(a) { Input::default() }
                else { binding.map(|b| b.input).unwrap_or_else(|| {
                    if self.simulation.player_count() == 1 { self.input } else { Input::default() }
                }) };
            Some(json!({"actor":entity.id,"instance":reference,
                "control":binding.map(|b| self.control_token(b.control)),"input":input,
                "life":self.simulation.vitality.state.lives[a],"dead":self.simulation.vitality.dead(a),
                "motion":entity.locomotion.unwrap_or_default().profile(input.mode)}))
        }).collect();
        json!({"tick":self.simulation.tick,"players":players,"recorded_events":self.control_events.len(),
            "legacy_input_available":self.simulation.player_count() <= 1 && self.controls.state.bindings.is_empty(),
            "limits":{"actors":crate::targeting::MAX_ACTORS,"npc_capacity":crate::templates::capacity(&self.scene),
                "events":self.recording().then_some(crate::controls::MAX_EVENTS),"events_per_tick":crate::controls::PER_TICK}})
    }
    fn actor_slot(&self, id: &str) -> Result<u16> {
        if let Some(token) = id.strip_prefix("ignis:") {
            let (scope, serial) = token
                .split_once(':')
                .ok_or_else(|| invalid("Malformed instance reference"))?;
            if scope != self.execution_scope
                || serial.len() != 16
                || !serial
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(invalid("Stale or malformed instance reference"));
            }
            let instance = u64::from_str_radix(serial, 16)
                .map_err(|_| invalid("Malformed instance reference"))?;
            return self
                .simulation
                .actors
                .slot_for_instance(instance)
                .ok_or_else(|| invalid("Unknown actor instance"));
        }
        self.simulation
            .actor_slot_by_id(id)
            .ok_or_else(|| invalid("Unknown actor ID"))
    }
    fn queue_ability(
        &mut self,
        actor: u16,
        operation: crate::abilities::Operation,
    ) -> Result<Value> {
        if self.recording() && self.ability_inputs.len() >= crate::abilities::MAX_INPUTS
            || self.ability_inputs.len() - self.consumed_ability_inputs
                >= crate::abilities::input_capacity(self.simulation.actors.actors.len())
            || self.admission.ability_counts[actor as usize] as usize
                >= crate::abilities::PER_ACTOR_PER_TICK
            || !self.clock_open()
        {
            return Err(invalid("Ability input budget exceeded"));
        }
        let target = match operation {
            crate::abilities::Operation::Activate { target, .. } => Some(target.unwrap_or(actor)),
            _ => None,
        };
        let entry = crate::abilities::Entry {
            life: crate::abilities::LifeStamp {
                source_instance: self.simulation.actors.reference(actor),
                target_instance: target.map(|t| self.simulation.actors.reference(t)),
                actor: self.simulation.vitality.state.lives[actor as usize],
                target: target.map(|t| self.simulation.vitality.state.lives[t as usize]),
            },
            tick: self.simulation.tick,
            sequence: self.admission.ability,
            actor,
            operation,
        };
        self.simulation
            .abilities
            .validate_entry(entry, &self.simulation.actors)?;
        let next = self
            .admission
            .ability
            .checked_add(1)
            .ok_or_else(|| invalid("Ability sequence exhausted"))?;
        self.ability_inputs.push(entry);
        self.admission.ability = next;
        self.admission.ability_counts[actor as usize] += 1;
        Ok(json!({"request_id":entry.sequence,"tick":entry.tick,"queued":true}))
    }
    fn inspect_abilities(&self) -> Value {
        let mut value = self.simulation.abilities.inspect(
            self.simulation.definitions(),
            &self.simulation.actors,
            self.simulation.tick,
        );
        value["pending_inputs"] = json!(self.ability_inputs[self.consumed_ability_inputs..].iter().map(|e|{
            let operation=match e.operation{crate::abilities::Operation::Activate{slot,target}=>json!({"kind":"activate","slot":slot,"target":target.and(e.life.target_instance).and_then(|r|self.simulation.actors.label(self.simulation.definitions(),r))}),crate::abilities::Operation::Aim{slot,aim}=>json!({"kind":"aim","slot":slot,"aim":aim}),crate::abilities::Operation::Cancel{execution_id}=>json!({"kind":"cancel","execution_id":execution_id})};
            json!({"tick":e.tick,"request_id":e.sequence,"actor":self.simulation.actors.label(self.simulation.definitions(),e.life.source_instance),"operation":operation,"life":e.life})
        }).collect::<Vec<_>>());
        value["tick"] = json!(self.simulation.tick);
        value["input_limits"] = json!({"per_actor_per_tick":crate::abilities::PER_ACTOR_PER_TICK,"per_tick":crate::abilities::input_capacity(self.simulation.actors.actors.len()),"recorded_total":self.recording().then_some(crate::abilities::MAX_INPUTS)});
        value
    }
    pub(crate) fn runtime_scope(&self) -> &str {
        &self.execution_scope
    }
    fn player_view_actor(&self) -> Option<usize> {
        let view = self.player_view.as_ref()?;
        let actor = self.simulation.actors.actor_slot(view.reference)?;
        (self.simulation.vitality.state.lives[actor] == view.life
            && !self.simulation.vitality.dead(actor))
        .then_some(actor)
    }
    pub fn player_camera(&self) -> Option<Camera> {
        self.resolved_player_view().map(|v| v.camera)
    }
    fn resolved_player_view(&self) -> Option<crate::player_view::Resolved> {
        let actor = self.player_view_actor()?;
        let e =
            &self.simulation.definitions().entities[self.simulation.actors.entity(actor as u16)];
        let config = &self.player_view.as_ref()?.config;
        let placement = config.placement(self.simulation.position(&e.id)?, e.transform.size[1]);
        let (fraction, overlap) = if config.mode == crate::player_view::Mode::ThirdPerson {
            self.simulation.camera_fraction(
                &e.id,
                placement.pivot,
                placement.desired,
                config.third_person.collision_radius + config.third_person.collision_padding,
            )
        } else {
            (1., false)
        };
        Some(config.resolve(placement, fraction, overlap))
    }
    pub(crate) fn player_view_context(&self) -> Option<(u64, u64, u64, u64)> {
        self.player_view_actor()?;
        let v = self.player_view.as_ref()?;
        Some((self.revision, v.reference.instance, v.life, v.serial))
    }
    pub(crate) fn player_view_config(&self) -> Option<crate::player_view::Config> {
        self.player_view_actor()?;
        Some(self.player_view.as_ref()?.config.clone())
    }
    fn inspect_player_view(&self) -> Value {
        let resolved = self.resolved_player_view();
        json!({"active":self.player_view_actor().is_some(),"view":self.player_view.as_ref().map(|v|&v.config),
            "camera":resolved.as_ref().map(|v|&v.camera),"resolved":resolved,"tick":self.simulation.tick,"session_only":true})
    }
    pub fn execute(&mut self, request: Request) -> Result<Value> {
        self.guard(request.expected_revision)?;
        if let Command::Wind { operation } = &request.command {
            return self.wind_command(operation);
        }
        if let Command::Particles { operation } = &request.command {
            return self.particles_command(operation);
        }
        if self.planet_active()
            && matches!(
                &request.command,
                Command::Undo | Command::Redo | Command::EditSceneLive { .. }
            )
        {
            return Err(invalid(
                "Disable planet runtime before editing authoring state",
            ));
        }
        if self.planet_view.is_some()
            && matches!(
                &request.command,
                Command::SelectAt { .. } | Command::PickEditorMesh { .. }
            )
        {
            return Err(invalid(
                "Authoring picking is unavailable in the session-only planet view",
            ));
        }
        match request.command {
            Command::SetPlanetView { view } => {
                self.set_planet_view(view)?;
                return Ok(self.inspect_planet_view());
            }
            Command::InspectPlanetView => return Ok(self.inspect_planet_view()),
            Command::FramePlanetView => {
                let mut view = self.planet_view_settings()?;
                let doc = &self.simulation.planet.as_ref().unwrap().config.document;
                let (_, height) =
                    crate::planet::orbit::height_bounds(doc, glam::DVec3::Y, std::f64::consts::PI);
                let outer = doc.radius() + height.max(view.atmosphere_height_meters);
                let half = (view.fov_degrees.to_radians() * 0.5).tan();
                let angle = half
                    .min(half * f64::from(view.viewport[0]) / f64::from(view.viewport[1]))
                    .atan();
                view.distance_meters = doc.radius() + outer / angle.sin() * 1.15;
                view.tilt_degrees = 0.;
                self.set_planet_view(Some(view))?;
                return Ok(self.inspect_planet_view());
            }
            Command::ZoomPlanetView { factor } => {
                if !factor.is_finite() || factor <= 0. {
                    return Err(invalid("Zoom requires a finite positive distance ratio"));
                }
                let mut view = self.planet_view_settings()?;
                view.distance_meters = if view.distance_meters == 0. && factor > 1. {
                    0.5
                } else {
                    ((view.distance_meters + 1.) * factor - 1.).max(0.)
                };
                self.set_planet_view(Some(view))?;
                return Ok(self.inspect_planet_view());
            }
            Command::LookPlanetView {
                yaw_delta_degrees,
                tilt_delta_degrees,
            } => {
                if !yaw_delta_degrees.is_finite() || !tilt_delta_degrees.is_finite() {
                    return Err(invalid("Planet look requires finite deltas"));
                }
                let mut view = self.planet_view_settings()?;
                view.yaw_degrees = (view.yaw_degrees + yaw_delta_degrees).rem_euclid(360.);
                view.tilt_degrees = (view.tilt_degrees + tilt_delta_degrees).clamp(-85., 165.);
                self.set_planet_view_paced(Some(view), true)?;
                return Ok(self.inspect_planet_view());
            }
            Command::SetPlanetViewport { width, height } => {
                self.prepare_planet_viewport(width, height)?;
                return Ok(self.inspect_planet_view());
            }
            Command::SetPlanetRuntime { config } => {
                if config.is_some() && !self.scene.wind.is_empty() {
                    return Err(invalid(
                        "Wind is not yet integrated with the planetary adapter",
                    ));
                }
                if self.planet_view.is_some() {
                    return Err(invalid(
                        "Disable planet view before replacing its source runtime",
                    ));
                }
                if self.playing {
                    return Err(invalid("Pause before changing planet runtime"));
                }
                if let Some(config) = config {
                    if !self.scene.particles.instances.is_empty() {
                        return Err(invalid(
                            "Remove particle placements before enabling the planetary adapter",
                        ));
                    }
                    self.enable_planet_runtime(*config)?;
                } else {
                    self.simulation.planet = None;
                    self.reset();
                }
                self.input = Input::default();
                self.input_revision += 1;
                self.view_revision += 1;
                return Ok(self.inspect_planet_runtime());
            }
            Command::InspectPlanetRuntime => return Ok(self.inspect_planet_runtime()),
            Command::SetMeshLodSettings { settings } => {
                settings.validate()?;
                self.mesh_lod_settings = settings;
                self.view_revision += 1;
                return Ok(
                    json!({"settings":settings,"experimental":true,"image_error_is_not_certified":true}),
                );
            }
            Command::InspectMeshLods => {
                return Ok(
                    json!({"settings":self.mesh_lod_settings,"compiled":self.mesh_lods.inspect(),"session_only":true}),
                );
            }
            Command::SetPlayerView { view } => {
                if self.planet_view.is_some() {
                    return Err(invalid(
                        "Disable planet view before replacing the player camera",
                    ));
                }
                if self.planet_active() && view.is_none() {
                    return Err(invalid(
                        "Planet runtime requires a player camera inside its coverage envelope",
                    ));
                }
                let next = if let Some(mut config) = view {
                    config.validate()?;
                    let actor = self.actor_slot(&config.actor)?;
                    let e = &self.simulation.definitions().entities
                        [self.simulation.actors.entity(actor)];
                    if e.body != Some(BodyKind::Player)
                        || self.simulation.vitality.dead(actor as usize)
                        || config.eye_height >= e.transform.size[1]
                    {
                        return Err(invalid(
                            "Player view requires a living player and eyes inside its height",
                        ));
                    }
                    config.yaw_degrees = config.yaw_degrees.rem_euclid(360.);
                    let reference = self.simulation.actors.reference(actor);
                    let life = self.simulation.vitality.state.lives[actor as usize];
                    let serial = match &self.player_view {
                        Some(v) if v.reference == reference && v.life == life => v.serial,
                        _ => self
                            .player_view_serial
                            .checked_add(1)
                            .ok_or_else(|| invalid("View identity exhausted"))?,
                    };
                    Some(crate::player_view::State {
                        config,
                        reference,
                        life,
                        serial,
                    })
                } else {
                    None
                };
                if let Some(v) = &next {
                    self.player_view_serial = v.serial;
                }
                self.player_view = next;
                self.view_revision += 1;
                return Ok(self.inspect_player_view());
            }
            Command::InspectPlayerView => return Ok(self.inspect_player_view()),
            Command::LookPlayerView {
                yaw_delta_degrees,
                pitch_delta_degrees,
            } => {
                if self.planet_view.is_some() {
                    return self.execute(
                        Command::LookPlanetView {
                            yaw_delta_degrees: f64::from(yaw_delta_degrees),
                            tilt_delta_degrees: f64::from(pitch_delta_degrees),
                        }
                        .into(),
                    );
                }
                if !yaw_delta_degrees.is_finite() || !pitch_delta_degrees.is_finite() {
                    return Err(invalid("Player look requires finite angle deltas"));
                }
                self.player_view_actor()
                    .ok_or_else(|| invalid("No current player view"))?;
                let config = &mut self.player_view.as_mut().unwrap().config;
                config.yaw_degrees = (config.yaw_degrees + yaw_delta_degrees).rem_euclid(360.);
                config.pitch_degrees =
                    (config.pitch_degrees + pitch_delta_degrees).clamp(-85., 85.);
                self.view_revision += 1;
                return Ok(
                    json!({"view_revision":self.view_revision,"yaw_degrees":config.yaw_degrees,"pitch_degrees":config.pitch_degrees}),
                );
            }
            Command::SetPlayerViewInput { input, control } => {
                Self::validate_input(input)?;
                self.player_view_actor()
                    .ok_or_else(|| invalid("No current player view"))?;
                let v = self.player_view.as_ref().unwrap();
                let input = if let Some(view) = &self.planet_view {
                    let mut config = v.config.clone();
                    config.yaw_degrees = view.settings.yaw_degrees as f32;
                    config.world_input(input)
                } else {
                    v.config.world_input(input)
                };
                let command = if let Some(control) = control {
                    let id = self.control_id(&control)?;
                    if self.controls.binding(id)?.actor != v.reference {
                        return Err(invalid("Control does not own the viewed player"));
                    }
                    Command::SetControlInput { control, input }
                } else {
                    Command::SetInput { input }
                };
                return self.execute(command.into());
            }
            Command::PlanMeshSweep { sweeps } => {
                return Ok(json!({"plan":crate::mesh_sweep::plan(&sweeps)?}));
            }
            Command::PreviewMeshMaterialFinish {
                source,
                plan,
                options,
            } => {
                return Ok(json!(crate::mesh_material_finish::finish(
                    &source, &plan, &options
                )?));
            }
            Command::InspectMeshMaterialDomain {
                asset,
                faces,
                options,
            } => {
                let report = crate::mesh_material_domain::inspect(
                    &self.scene.meshes.asset(&asset)?.geometry,
                    faces.as_deref(),
                    &options,
                )?;
                return Ok(
                    json!({"asset":asset,"reference":self.mesh_reference(),"report":report}),
                );
            }
            Command::PreviewMeshMaterialDomain {
                source,
                faces,
                options,
            } => {
                return Ok(json!({"report":crate::mesh_material_domain::inspect(
                    &source, faces.as_deref(), &options,
                )?}));
            }
            Command::PreviewMeshSurface {
                source,
                vertices,
                anchors,
                options,
            } => {
                return Ok(json!(crate::mesh_surface::smooth_region(
                    &source, &vertices, &anchors, &options
                )?));
            }
            Command::PreviewMeshRingLoft { source, lofts } => {
                return Ok(json!(crate::mesh_ring_loft::fit(&source, &lofts)?));
            }
            Command::InspectMeshIslands { asset } => {
                return Ok(
                    json!({"asset":asset,"reference":self.mesh_reference(),"topology":self.scene.meshes.islands(&asset)?}),
                );
            }
            Command::PreviewMeshEdit { edit } => {
                let (scene, report) = self.prepare_mesh_edit(&edit)?;
                let mut data = json!({"meshes":scene.meshes,"plants":scene.plants});
                if let Some(report) = report {
                    data["modeling_report"] = json!(report);
                }
                return Ok(data);
            }
            Command::PreviewMeshJunctionUnion {
                source,
                ports,
                root,
                options,
            } => {
                return Ok(json!(crate::mesh_junction_union::fit(
                    &source,
                    &ports,
                    root.as_deref(),
                    &options
                )?));
            }
            Command::InspectMeshBoundaries { asset } => {
                let boundaries =
                    crate::mesh_bridge::boundaries(&self.scene.meshes.asset(&asset)?.geometry)?;
                return Ok(
                    json!({"asset":asset,"reference":self.mesh_reference(),"boundaries":boundaries}),
                );
            }
            Command::PreviewMeshJunction {
                collars,
                options,
                fusion,
            } => {
                let result = match fusion {
                    Some(fusion) => crate::mesh_organic::fused_junction(&collars, options, fusion)?,
                    None => crate::mesh_organic::junction(&collars, options)?,
                };
                return Ok(json!(result));
            }
            Command::InspectMeshTopology { asset } => {
                let topology = crate::mesh_topology::Topology::inspect(
                    &self.scene.meshes.asset(&asset)?.geometry,
                )?;
                return Ok(
                    json!({"asset":asset,"reference":self.mesh_reference(),"topology":topology}),
                );
            }
            Command::InspectMeshSelection => {
                return Ok(
                    json!({"selection":self.mesh_selection,"view_revision":self.view_revision}),
                );
            }
            Command::SelectMeshComponents { selection } => {
                let selection = selection
                    .map(|mut selected| {
                        self.guard_mesh_reference(&selected.reference)?;
                        let topology = crate::mesh_topology::Topology::inspect(
                            &self.scene.meshes.asset(&selected.asset)?.geometry,
                        )?;
                        selected.components = topology.canonicalize(&selected.components)?;
                        Ok(selected)
                    })
                    .transpose()?;
                if selection != self.mesh_selection {
                    self.mesh_selection = selection;
                    self.view_revision += 1;
                }
                return Ok(
                    json!({"selection":self.mesh_selection,"view_revision":self.view_revision}),
                );
            }
            Command::EditPlant { edit } => {
                self.apply_edits(std::iter::once(SceneEdit::Plant { edit }))?
            }
            Command::PreviewPlant { edit } => {
                let mut scene = self.scene.clone();
                scene.plants.apply(edit, &mut scene.meshes)?;
                scene.plants.rebuild_connections(&mut scene.meshes)?;
                scene.validate()?;
                return Ok(json!({"plants":scene.plants,"assets":scene.meshes.assets}));
            }
            Command::InspectPlants => {
                return Ok(
                    json!({"plants":self.scene.plants.entries.iter().map(|e|&e.definition).collect::<Vec<_>>()}),
                );
            }
            Command::InspectPlant { id } => {
                let entry = self.scene.plants.entry(&id)?;
                let evaluated = entry.evaluate()?;
                return Ok(json!({"plant":entry,"report":evaluated.report}));
            }
            Command::InspectPlantSampling { id } => {
                let entry = self.scene.plants.entry(&id)?;
                return Ok(json!({"id":id,"revision":self.revision,
                    "sampling":entry.definition.sample()?}));
            }
            Command::InspectPlantConnections => {
                return Ok(json!({"connections":self.scene.plants.connections,
                    "reports":self.scene.plants.connection_reports(&self.scene.meshes)?}));
            }
            Command::EditMesh { edit } => {
                if matches!(&edit, crate::meshes::Edit::ModelSelection { .. }) {
                    let (scene, report) = self.prepare_mesh_edit(&edit)?;
                    self.replace(scene)?;
                    return Ok(
                        json!({"revision":self.revision,"tick":self.simulation.tick,"playing":self.playing,"modeling_report":report}),
                    );
                }
                self.apply_edits(std::iter::once(SceneEdit::Mesh { edit }))?
            }
            Command::InspectMeshObjects { object } => {
                return crate::mesh_objects::inspect(&self.scene.meshes, object.as_deref());
            }
            Command::InspectMeshes => {
                return Ok(
                    serde_json::json!({"tick":self.simulation.tick,"limits":{"assets":null,"instances":null,"asset_vertices":u32::MAX,"asset_triangles":u32::MAX,"total_vertices":null,"total_triangles":null,"draw_triangles":null,"basis":"host memory and active device resource limits; u32 mesh indices"},"assets":self.scene.meshes.assets.iter().map(|a|serde_json::json!({"id":a.id,"vertices":a.geometry.positions.len(),"triangles":a.geometry.triangles.len(),"bones":a.rig.as_ref().map_or(0,|r|r.bones.len()),"clips":a.clips.iter().map(|c|&c.id).collect::<Vec<_>>()})).collect::<Vec<_>>(),"objects":self.scene.meshes.objects,"instances":self.scene.meshes.instances}),
                );
            }
            Command::ProjectLandmarks {
                width,
                height,
                camera,
                landmarks,
            } => {
                return Ok(json!(crate::landmark_projection::inspect(
                    camera.as_ref().unwrap_or(&self.scene.camera),
                    width,
                    height,
                    &landmarks,
                )?));
            }
            Command::InspectMesh { asset } => {
                return Ok(serde_json::json!({"asset":self.scene.meshes.asset(&asset)?}));
            }
            Command::InspectMeshNormals {
                instance,
                tick,
                faces,
                sample_limit,
            } => {
                let report = crate::mesh_normals::inspect(
                    &self.scene.meshes,
                    &instance,
                    tick.unwrap_or(self.simulation.tick),
                    faces.as_deref(),
                    sample_limit,
                )?;
                return Ok(
                    json!({"instance":instance,"reference":self.mesh_reference(),"report":report}),
                );
            }
            Command::InspectMeshPose { instance, tick } => {
                if tick.is_some_and(|t| t > MAX_TICKS as u64) {
                    return Err(invalid("Mesh pose query tick must be <=36000"));
                }
                return Ok(
                    serde_json::json!({"pose":self.scene.meshes.pose(&instance,tick.unwrap_or(self.simulation.tick))?}),
                );
            }

            Command::InspectMeshCollisions => return Ok(crate::surfaces::inspect(&self.scene)),
            Command::Inspect => return Ok(self.snapshot()),
            Command::PlanWorldRegions { query } => {
                let plan = crate::world_regions::plan(*query).map_err(invalid)?;
                return serde_json::to_value(plan).map_err(|e| invalid(e.to_string()));
            }
            Command::QueryTerrain { query } => return crate::terrain::query(*query),
            Command::QueryLandform { query } => return crate::landforms::query(*query),
            Command::PublishLandform { publication } => {
                if self.playing {
                    return Err(invalid("Pause before publishing a landform"));
                }
                let collision = publication.collision;
                let (asset, instance, compiled) = crate::landforms::publication(*publication)?;
                let instance_id = instance.id.clone();
                let mut edits = vec![
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutAsset { asset },
                    },
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutInstance { instance },
                    },
                ];
                if let Some(enabled) = collision {
                    edits.push(SceneEdit::SetMeshCollision {
                        instance: instance_id.clone(),
                        enabled,
                    });
                }
                self.apply_edits(edits.into_iter())?;
                return Ok(
                    json!({"revision":self.revision,"source_id":compiled.source_id,"source_revision":compiled.source_revision,"source_sha256":compiled.source_sha256,"parts":compiled.parts,"visual_only":!self.scene.collision_meshes.contains(&instance_id),"source_embedded_in_scene":false,"boolean_union":false}),
                );
            }
            Command::PlanTerrainCoverage { query } => {
                return serde_json::to_value(crate::terrain::coverage::plan(*query)?)
                    .map_err(|e| invalid(e.to_string()));
            }
            Command::PublishTerrainRegion { publication } => {
                if self.playing {
                    return Err(invalid("Pause before publishing terrain region"));
                }
                let collision = publication.collision;
                let (asset, instance, compiled) =
                    crate::terrain::region::publication(*publication)?;
                let id = instance.id.clone();
                let mut edits = vec![
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutAsset { asset },
                    },
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutInstance { instance },
                    },
                ];
                if let Some(enabled) = collision {
                    edits.push(SceneEdit::SetMeshCollision {
                        instance: id.clone(),
                        enabled,
                    });
                }
                self.apply_edits(edits.into_iter())?;
                return Ok(
                    json!({"revision":self.revision,"terrain":compiled.report(),"visual_only":!self.scene.collision_meshes.contains(&id)}),
                );
            }
            Command::PublishTerrain { publication } => {
                if self.playing {
                    return Err(invalid("Pause before publishing terrain"));
                }
                let collision = publication.collision;
                let terrain_format = publication.document.version;
                let (asset, instance, compiled) = crate::terrain::publication(*publication)?;
                let instance_id = instance.id.clone();
                let mut edits = vec![
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutAsset { asset },
                    },
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutInstance { instance },
                    },
                ];
                if let Some(enabled) = collision {
                    edits.push(SceneEdit::SetMeshCollision {
                        instance: instance_id.clone(),
                        enabled,
                    });
                }
                self.apply_edits(edits.into_iter())?;
                return Ok(
                    json!({"revision":self.revision,"source_id":compiled.source_id,"source_revision":compiled.source_revision,"origin":compiled.origin,"sampled_error_meters":compiled.sampled_error_meters,"visual_only":!self.scene.collision_meshes.contains(&instance_id),"source_embedded_in_scene":false,"terrain_format":terrain_format}),
                );
            }
            Command::QueryPlanet { query } => return crate::planet::query(*query),
            Command::PublishPlanet { publication } => {
                if self.playing {
                    return Err(invalid("Pause before publishing a planet"));
                }
                let collision = publication.collision;
                let (asset, instance, compiled) = crate::planet::mesh::publication(*publication)?;
                let instance_id = instance.id.clone();
                let mut edits = vec![
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutAsset { asset },
                    },
                    SceneEdit::Mesh {
                        edit: crate::meshes::Edit::PutInstance { instance },
                    },
                ];
                if let Some(enabled) = collision {
                    edits.push(SceneEdit::SetMeshCollision {
                        instance: instance_id.clone(),
                        enabled,
                    });
                }
                self.apply_edits(edits.into_iter())?;
                return Ok(
                    json!({"revision":self.revision,"planet":compiled.report,"visual_only":!self.scene.collision_meshes.contains(&instance_id)}),
                );
            }
            Command::QueryWorldSpace { query } => {
                let answer =
                    crate::world_space::query(query).map_err(|e| invalid(e.to_string()))?;
                return serde_json::to_value(answer).map_err(|e| invalid(e.to_string()));
            }
            Command::InspectCheckpoint => {
                return Ok(
                    json!({"format":crate::checkpoint::VERSION,"max_bytes":crate::checkpoint::MAX_BYTES,"native_physics":"rapier3d-0.30.1","platform":crate::checkpoint::platform(),"baseline_tick":self.baseline_tick,"tick":self.simulation.tick,"baseline_bytes":self.baseline.as_ref().map_or(0,|b|b.len()),"mode":self.execution_mode}),
                );
            }
            Command::InspectExecution => return Ok(self.inspect_execution()),
            Command::SetExecutionMode { mode } => {
                self.set_execution_mode(mode)?;
                return Ok(self.inspect_execution());
            }
            Command::QueryGroundTraversal { entity, start, end } => {
                if self.planet_active() {
                    return Err(invalid(
                        "Planet global navigation has not migrated to transported coordinates",
                    ));
                }
                if start
                    .iter()
                    .chain(end.iter())
                    .any(|v| !v.is_finite() || v.abs() > 10000.)
                    || glam::Vec3::from(start).distance(glam::Vec3::from(end)) > 32.
                {
                    return Err(invalid(
                        "Ground traversal requires finite feet positions within +/-10000 and distance <=32m",
                    ));
                }
                let e = self
                    .simulation
                    .definitions()
                    .entities
                    .iter()
                    .find(|e| e.id == entity && self.simulation.position(&e.id).is_some())
                    .ok_or_else(|| invalid("Unknown present ground actor"))?;
                if e.character_controller.is_none() {
                    return Err(invalid("Ground traversal requires character_controller"));
                }
                let report = self.simulation.mobility.ground.as_ref().map_or_else(
                    || {
                        crate::ground_navigation::World::new(self.simulation.definitions())
                            .trace(e, start, end)
                    },
                    |w| w.trace(e, start, end),
                );
                return Ok(json!({"tick":self.simulation.tick,"static_only":true,"report":report}));
            }
            Command::InspectCharacterControllers => {
                let configs: std::collections::BTreeMap<_, _> = self
                    .simulation
                    .character_controllers
                    .iter()
                    .map(|(id, state)| {
                        (
                            id,
                            self.simulation.definitions().entities[state.entity_index as usize]
                                .character_controller
                                .unwrap(),
                        )
                    })
                    .collect();
                return Ok(
                    json!({"tick":self.simulation.tick,"controllers":self.simulation.character_controllers,"configs":configs}),
                );
            }
            Command::InspectInstances => return Ok(self.inspect_instances()),
            Command::InspectControls => return Ok(self.inspect_controls()),
            Command::BindControl { actor } => {
                let actor = self.actor_slot(&actor)?;
                return self.record_control(crate::controls::Operation::Bind {
                    control: self.controls.state.next_control,
                    actor: self.simulation.actors.reference(actor),
                    life: self.simulation.vitality.state.lives[actor as usize],
                });
            }
            Command::SetControlInput { control, input } => {
                let control = self.control_id(&control)?;
                let binding = self.controls.binding(control)?;
                let actor = self
                    .simulation
                    .actors
                    .actor_slot(binding.actor)
                    .ok_or_else(|| invalid("Unknown controlled instance"))?;
                return self.record_control(crate::controls::Operation::Input {
                    control,
                    input,
                    life: self.simulation.vitality.state.lives[actor],
                });
            }
            Command::ReleaseControl { control } => {
                let control = self.control_id(&control)?;
                return self.record_control(crate::controls::Operation::Release { control });
            }
            Command::InspectWorld => {
                return Ok(
                    json!({"tick":self.simulation.tick,"authored_entities":self.simulation.definitions().entities.len(),"positions":self.simulation.positions(),"dynamic_population":true,"spawn_supported":true,"despawn_supported":true}),
                );
            }
            Command::InspectOrigins => return Ok(self.simulation.abilities.inspect_origins()),
            Command::InspectGenerators => return Ok(self.inspect_generators()),
            Command::ControlNpcGenerator { generator, action } => {
                let index = self
                    .scene
                    .npc_generators
                    .iter()
                    .position(|g| g.id == generator)
                    .ok_or_else(|| invalid("Unknown NPC generator"))?;
                return self.queue_population(crate::population::Operation::ControlGenerator {
                    generator: index as u8,
                    action,
                });
            }
            Command::SetInteractions { document } => {
                self.apply_edits(std::iter::once(SceneEdit::SetInteractions { document }))?
            }
            Command::SetNpcGenerators { generators } => {
                self.apply_edits(std::iter::once(SceneEdit::SetNpcGenerators { generators }))?
            }
            Command::InspectTemplates => {
                return Ok(
                    json!({"templates":self.scene.npc_templates,"maximum":crate::templates::MAX_TEMPLATES}),
                );
            }
            Command::QueryGeometry {
                entity,
                start,
                end,
                half_extent,
                interior,
            } => {
                crate::spawn_support::validate_position(start)?;
                crate::spawn_support::validate_position(end)?;
                if half_extent
                    .iter()
                    .any(|h| !h.is_finite() || *h < 0. || *h > 500.)
                {
                    return Err(invalid("Query half_extent must be finite in [0,500]"));
                }
                let e = self
                    .simulation
                    .definitions()
                    .entities
                    .iter()
                    .find(|e| e.id == entity)
                    .ok_or_else(|| invalid("Unknown geometry entity"))?;
                let position = self
                    .simulation
                    .position(&entity)
                    .ok_or_else(|| invalid("Geometry entity is not present"))?;
                let a = glam::Vec3::from(start);
                let delta = glam::Vec3::from(end) - a;
                let fraction = crate::geometry::Volume::entity(e, position.into()).cast(
                    a,
                    delta,
                    half_extent.into(),
                    interior,
                    0.,
                );
                return Ok(
                    json!({"tick":self.simulation.tick,"geometry":e.geometry,"entity":entity,"fraction":fraction,"position":fraction.map(|t|(a+delta*t).to_array()),"geometry_only":true}),
                );
            }
            Command::QuerySpawnSupport {
                template,
                position,
                max_drop,
            } => {
                crate::spawn_support::validate_position(position)?;
                let projection = crate::spawn_support::Projection { max_drop };
                projection.validate()?;
                let index = self
                    .scene
                    .npc_templates
                    .iter()
                    .position(|t| t.id == template)
                    .ok_or_else(|| invalid("Unknown NPC template"))?;
                let report = self
                    .simulation
                    .query_spawn_support(index as u8, position, projection);
                return Ok(
                    json!({"tick":self.simulation.tick,"template":template,"origin":position,"max_drop":max_drop,"geometry_only":true,"report":report,"surface_id":report.surface.map(|i| self.simulation.definitions().entities[i as usize].id.as_str()).or_else(||report.mesh_surface.map(|i| self.simulation.definitions().collision_meshes[i as usize].as_str()))}),
                );
            }
            Command::SpawnActor {
                template,
                position,
                target,
            } => return self.queue_spawn(&template, position, target.as_deref()),
            Command::SetNpcTemplates { templates } => {
                self.apply_edits(std::iter::once(SceneEdit::SetNpcTemplates { templates }))?
            }
            Command::InspectPopulation => return Ok(self.inspect_population()),
            Command::SetNpcCapacity { capacity } => {
                self.apply_edits(std::iter::once(SceneEdit::SetNpcCapacity { capacity }))?
            }
            Command::SetAvatarAudio { audio } => {
                self.apply_edits(std::iter::once(SceneEdit::SetAvatarAudio { audio }))?
            }
            Command::InspectNpcPresentation => {
                return Ok(
                    json!({"tick":self.simulation.tick,"state":self.simulation.presentation,"poses":self.simulation.presentation_poses(),"omitted_visuals":self.render_frame().meshes.omitted_avatars,"audio":{"rate":crate::npc_presentation::RATE,"voices":crate::npc_presentation::VOICES,"device_output":false}}),
                );
            }
            Command::InspectInteractions => {
                return Ok(
                    json!({"tick":self.simulation.tick,"document":self.simulation.interactions.document,"state":self.simulation.interactions.state}),
                );
            }
            Command::InspectNpcRoutines => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)| {
                    let config=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap().npc.as_ref().unwrap();
                    (id.clone(),json!({"configuration":config.routine,"profile":state.mind_profile,"state":state.routine,"navigation":self.simulation.navigation[id]}))
                }).collect();
                return Ok(
                    json!({"tick":self.simulation.tick,"npcs":npcs,"automatic_actions_per_tick":16}),
                );
            }
            Command::InspectDialogue { actor } => {
                let actor = self.actor_slot(&actor)?;
                let mut result = self.simulation.inspect_dialogue(actor);
                if let Some(serial) = result["session"]["serial"].as_u64() {
                    result["handle"] = json!(format!(
                        "ignis-dialogue:{}:{serial:016x}",
                        self.execution_scope
                    ));
                }
                return Ok(result);
            }
            Command::Interact {
                actor,
                point,
                action,
            } => {
                let actor = self.actor_slot(&actor)?;
                let runtime = &self.simulation.interactions;
                let p = runtime
                    .document
                    .points
                    .iter()
                    .position(|p| p.id == point)
                    .ok_or_else(|| invalid("Unknown interaction point"))?;
                let action = runtime
                    .document
                    .actions
                    .iter()
                    .position(|a| a.id == action)
                    .ok_or_else(|| invalid("Unknown interaction action"))?
                    as u8;
                let instance = runtime.state.anchors[p]
                    .ok_or_else(|| invalid("Unavailable interaction anchor"))?;
                let life = self
                    .simulation
                    .actors
                    .actor_slot(instance)
                    .map(|a| self.simulation.vitality.state.lives[a]);
                return self.queue_interaction(
                    actor,
                    crate::interactions::Request::Use {
                        point: p as u8,
                        action,
                        instance,
                        life,
                    },
                );
            }
            Command::TalkNpc { actor, npc } => {
                let actor = self.actor_slot(&actor)?;
                let target = self.actor_slot(&npc)?;
                let entity =
                    &self.simulation.definitions().entities[self.simulation.actors.entity(target)];
                let id = entity
                    .npc
                    .as_ref()
                    .and_then(|n| n.dialogue.as_ref())
                    .ok_or_else(|| invalid("NPC has no dialogue"))?;
                let dialogue =
                    self.simulation
                        .interactions
                        .document
                        .dialogues
                        .iter()
                        .position(|d| d.id == *id)
                        .ok_or_else(|| invalid("Unknown dialogue"))? as u8;
                return self.queue_interaction(
                    actor,
                    crate::interactions::Request::Talk {
                        target,
                        instance: self.simulation.actors.reference(target),
                        life: self.simulation.vitality.state.lives[target as usize],
                        dialogue,
                    },
                );
            }
            Command::ChooseDialogue {
                actor,
                session,
                choice,
            } => {
                let actor = self.actor_slot(&actor)?;
                let serial = self.dialogue_id(&session)?;
                let session = self
                    .simulation
                    .interactions
                    .state
                    .sessions
                    .get(actor as usize)
                    .copied()
                    .flatten()
                    .filter(|s| s.serial == serial)
                    .ok_or_else(|| invalid("Stale dialogue session"))?;
                let node = &self.simulation.interactions.document.dialogues
                    [session.dialogue as usize]
                    .nodes[session.node as usize];
                let choice = node
                    .choices
                    .iter()
                    .position(|c| c.id == choice)
                    .ok_or_else(|| invalid("Unknown dialogue choice"))?
                    as u8;
                return self.queue_interaction(
                    actor,
                    crate::interactions::Request::Choose {
                        session: serial,
                        revision: session.revision,
                        choice,
                    },
                );
            }
            Command::CloseDialogue { actor, session } => {
                let actor = self.actor_slot(&actor)?;
                let session = self.dialogue_id(&session)?;
                return self
                    .queue_interaction(actor, crate::interactions::Request::Close { session });
            }
            Command::OrderNpc { actor, order } => {
                let actor = self.actor_slot(&actor)?;
                return self.queue_npc_orders(vec![actor], order);
            }
            Command::OrderNpcGroup { group, order } => {
                if group == 0 {
                    return Err(invalid("NPC group must be nonzero"));
                }
                let members = self
                    .simulation
                    .actors
                    .order()
                    .iter()
                    .copied()
                    .filter(|&a| {
                        let e = &self.simulation.definitions().entities
                            [self.simulation.actors.entity(a)];
                        !self.simulation.vitality.dead(a as usize)
                            && self
                                .simulation
                                .actors
                                .entity_reference(self.simulation.actors.entity(a))
                                .is_some()
                            && e.npc
                                .as_ref()
                                .and_then(|n| n.cooperation)
                                .is_some_and(|c| c.group == group)
                    })
                    .collect();
                return self.queue_npc_orders(members, order);
            }
            Command::InspectNpcCooperation => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)| {
                    let config=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap().npc.as_ref().unwrap();
                    (id.clone(),json!({"configuration":config.cooperation,"profile":state.mind_profile,"order":state.order,"navigation":self.simulation.navigation[id]}))
                }).collect();
                return Ok(
                    json!({"tick":self.simulation.tick,"npcs":npcs,"support_claims":self.simulation.abilities.state.support_claims,"support_decisions":self.simulation.abilities.state.support_coordination}),
                );
            }
            Command::DespawnActor { actor } => return self.queue_despawn(&actor),
            Command::InspectAttribute {
                entity,
                attribute,
                ability,
                tags,
            } => {
                let owner = self
                    .simulation
                    .definitions()
                    .entities
                    .iter()
                    .position(|e| e.id == entity)
                    .ok_or_else(|| invalid("Unknown entity"))? as u16;
                if self.simulation.position(&entity).is_none() {
                    return Err(invalid(
                        "Attribute owner is not present in the runtime world",
                    ));
                }
                let runtime = &self.simulation.abilities;
                let registry = &runtime.attributes;
                let key = registry
                    .key(&attribute)
                    .ok_or_else(|| invalid("Unknown attribute"))?;
                let definition = ability
                    .as_ref()
                    .map(|id| {
                        self.scene
                            .ability_catalog
                            .abilities
                            .iter()
                            .position(|a| a.id == *id)
                            .ok_or_else(|| invalid("Unknown context ability"))
                    })
                    .transpose()?;
                if tags.len() > 64 {
                    return Err(invalid("At most64 context tags"));
                }
                let mut context = definition.map_or_else(Default::default, |d| registry.context(d));
                if matches!(
                    key,
                    crate::attributes::DAMAGE_RECEIVED | crate::attributes::DAMAGE_DEALT
                ) {
                    context.tags = 0;
                }
                for t in tags {
                    context.tags |= 1u64
                        << registry
                            .tags
                            .iter()
                            .position(|s| *s == t)
                            .ok_or_else(|| invalid("Unknown context tag"))?;
                }
                let fallback = definition.map_or(registry.default_value(key), |i| {
                    let d = &self.scene.ability_catalog.abilities[i];
                    match key {
                        crate::attributes::INTERRUPTIBLE => {
                            crate::attributes::Value::boolean(d.interruptible)
                        }
                        crate::attributes::DISPELLABLE => {
                            crate::attributes::Value::boolean(d.dispellable)
                        }
                        _ => registry.default_value(key),
                    }
                });
                let mut result = registry.inspect_value(
                    &self.simulation.actors,
                    &runtime.state.buffs,
                    owner,
                    key,
                    fallback,
                    context,
                    self.simulation.tick,
                    self.simulation.definitions(),
                );
                result["ability"] = json!(ability);
                result["tags"] = json!(
                    registry
                        .tags
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| context.tags & (1u64 << i) != 0)
                        .map(|(_, t)| t)
                        .collect::<Vec<_>>()
                );
                result["tick"] = json!(self.simulation.tick);
                return Ok(result);
            }
            Command::InspectVitality => {
                return Ok(self
                    .simulation
                    .vitality
                    .inspect(self.simulation.definitions(), &self.simulation.actors));
            }
            Command::InspectAbilities | Command::InspectResources => {
                return Ok(self.inspect_abilities());
            }
            Command::SetAbilityCatalog { catalog } => {
                self.apply_edits(std::iter::once(SceneEdit::SetAbilityCatalog { catalog }))?
            }
            Command::ActivateAbility {
                actor,
                slot,
                target,
            } => {
                let a = self.actor_slot(&actor)?;
                let t = target
                    .as_deref()
                    .map(|id| self.actor_slot(id))
                    .transpose()?;
                return self
                    .queue_ability(a, crate::abilities::Operation::Activate { slot, target: t });
            }
            Command::AimAbility { actor, slot, aim } => {
                let a = self.actor_slot(&actor)?;
                return self.queue_ability(a, crate::abilities::Operation::Aim { slot, aim });
            }
            Command::AimAbilityAt {
                actor,
                slot,
                x,
                y,
                width,
                height,
                plane_y,
            } => {
                let a = self.actor_slot(&actor)?;
                let position = crate::picking::point_on_plane(
                    &self.scene.camera,
                    x,
                    y,
                    width,
                    height,
                    plane_y,
                )?;
                let mut result = self.queue_ability(
                    a,
                    crate::abilities::Operation::Aim {
                        slot,
                        aim: crate::abilities::Aim::Point { position },
                    },
                )?;
                result["aim_point"] = json!(position);
                return Ok(result);
            }
            Command::CancelAbility {
                actor,
                execution_id,
            } => {
                let a = self.actor_slot(&actor)?;
                return self.queue_ability(a, crate::abilities::Operation::Cancel { execution_id });
            }
            Command::InspectSelection => return Ok(self.inspect_selection()),
            Command::SelectEntity { id } => {
                let selection = id
                    .map(|id| {
                        if self.simulation.position(&id).is_none() {
                            return Err(invalid(
                                "Selection entity is absent from the runtime world",
                            ));
                        }
                        let index = self
                            .simulation
                            .definitions()
                            .entities
                            .iter()
                            .position(|e| e.id == id)
                            .ok_or_else(|| {
                                invalid(format!("Selection entity {id} does not exist"))
                            })?;
                        Ok(Selected {
                            id,
                            index: index as u16,
                        })
                    })
                    .transpose()?;
                self.set_selection(selection);
                return Ok(
                    json!({"selection":self.selected_id(),"view_revision":self.view_revision}),
                );
            }
            Command::SelectAt {
                x,
                y,
                width,
                height,
            } => {
                let hit = crate::picking::pick(
                    self.simulation.definitions(),
                    &self.simulation,
                    x,
                    y,
                    width,
                    height,
                )?;
                self.set_selection(hit.map(|h| {
                    Selected {
                        id: self.simulation.definitions().entities[h.entity_index as usize]
                            .id
                            .clone(),
                        index: h.entity_index,
                    }
                }));
                return Ok(
                    json!({"selection":self.selected_id(),"hit":hit,"view_revision":self.view_revision}),
                );
            }
            Command::SetDebugView {
                npc_vision,
                npc_hearing,
                noise_events,
                npc_routes,
            } => {
                let next = DebugView {
                    npc_vision: npc_vision.unwrap_or(self.debug_view.npc_vision),
                    npc_hearing: npc_hearing.unwrap_or(self.debug_view.npc_hearing),
                    noise_events: noise_events.unwrap_or(self.debug_view.noise_events),
                    npc_routes: npc_routes.unwrap_or(self.debug_view.npc_routes),
                };
                if self.debug_view != next {
                    self.debug_view = next;
                    self.view_revision += 1;
                }
                return Ok(
                    json!({"debug_view":self.debug_view,"view_revision":self.view_revision}),
                );
            }
            Command::InspectTriggers => {
                return Ok(
                    json!({"tick":self.simulation.tick,"triggers":self.simulation.triggers.inspect()}),
                );
            }
            Command::InspectPatrols => {
                return Ok(
                    json!({"tick":self.simulation.tick,"patrols":self.simulation.patrols,"routes":self.simulation.patrol_routes}),
                );
            }
            Command::InspectFootsteps => {
                return Ok(
                    json!({"tick":self.simulation.tick,"footsteps":self.simulation.footsteps,"recent_capacity":crate::footsteps::RECENT_CAPACITY}),
                );
            }
            Command::InspectNoises => {
                return Ok(
                    json!({"tick":self.simulation.tick,"noises":self.noises,"recent_manual":self.recent_manual_noises,
                    "ability_noises":self.simulation.abilities.state.noises,"recent_ability_noises":self.simulation.abilities.state.recent_noises,"ability_limits":{"per_source_per_tick":crate::abilities::NOISES_PER_SOURCE,"per_tick":crate::abilities::noise_capacity(self.simulation.actors.actors.len()),"recent_capacity":crate::abilities::RECENT_NOISES},
                    "footsteps":self.simulation.footsteps.recent,"pending_footsteps":self.simulation.footsteps.pending,"dropped_footsteps":self.simulation.footsteps.dropped_recent,"pending":self.noises.len()-self.consumed_noises,"capacity":self.recording().then_some(MAX_NOISES),"per_tick_capacity":MAX_NOISES_PER_TICK}),
                );
            }
            Command::EmitNoise { origin, radius } => {
                if self.recording() && self.noises.len() >= MAX_NOISES
                    || self.noises.len() - self.consumed_noises >= MAX_NOISES_PER_TICK
                    || !self.clock_open()
                {
                    return Err(invalid("Noise input budget exhausted"));
                }
                let position = match origin {
                    NoiseOrigin::Position { position } => position,
                    NoiseOrigin::Entity { id } => {
                        if let Some(index) = self
                            .simulation
                            .definitions()
                            .entities
                            .iter()
                            .position(|e| e.id == id)
                            && self
                                .simulation
                                .vitality
                                .dead(self.simulation.actors.slot(index) as usize)
                        {
                            return Err(invalid("Dead actor cannot emit noise"));
                        }
                        self.simulation.position(&id).ok_or_else(|| {
                            invalid("Noise source is not present in the runtime world")
                        })?
                    }
                };
                let next = self
                    .admission
                    .noise
                    .checked_add(1)
                    .filter(|&n| n <= crate::noise::MAX_SERIAL)
                    .ok_or_else(|| invalid("Noise identity exhausted"))?;
                let noise = Noise {
                    emitter_instance: None,
                    id: crate::noise::event_id(crate::noise::Domain::Manual, self.admission.noise),
                    tick: self.simulation.tick,
                    position,
                    radius,
                    emitter_index: None,
                };
                noise.validate()?;
                self.noises.push(noise);
                self.admission.noise = next;
                if self.debug_view.noise_events {
                    self.view_revision += 1;
                }
                return Ok(json!({"queued":noise,"consumed_on_next_step":true}));
            }
            Command::InspectRelations => {
                return Ok(self
                    .simulation
                    .actors
                    .inspect(self.simulation.definitions()));
            }
            Command::InspectTargeting => return Ok(self.inspect_targeting()),
            Command::InspectKnowledge => return Ok(self.inspect_knowledge()),
            Command::InspectTerritory => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)| {
                    let config=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap().npc.as_ref().unwrap();
                    (id.clone(),json!({"policy":config.territory,"decision_tick":state.decision_tick,"territory":state.territory,"position":self.simulation.position(id),"navigation":self.simulation.navigation[id]}))
                }).collect();
                return Ok(json!({"tick":self.simulation.tick,"npcs":npcs}));
            }
            Command::InspectNpcMind => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)| {
                    let config=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap().npc.as_ref().unwrap();
                    (id.clone(),json!({"configuration":config.mind,"trait_order":["anxiety","caution","concern","fear","courage","curiosity","aggression","sociability","discipline"],"profile":state.mind_profile,"decision":state.objectives.mind,"search_points":state.mind_search_points,"planning":self.simulation.mobility.states.get(id).and_then(|s|s.ground_planning)}))
                }).collect();
                return Ok(json!({"tick":self.simulation.tick,"npcs":npcs}));
            }
            Command::InspectNpcCirculation => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)| {
                    let config=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap().npc.as_ref().unwrap();
                    (id.clone(),json!({"configuration":config.circulation,"profile":state.mind_profile,"state":state.circulation}))
                }).collect();
                return Ok(json!({"tick":self.simulation.tick,"npcs":npcs}));
            }
            Command::InspectNpcTactics => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)|
                    (id.clone(),json!({"profile":state.mind_profile,"state":state.tactics,"navigation":self.simulation.navigation[id]}))).collect();
                return Ok(json!({"tick":self.simulation.tick,"npcs":npcs}));
            }
            Command::InspectObjectives => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)| {
                    let config=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap().npc.as_ref().unwrap();
                    (id.clone(),json!({"mind":config.mind,"policy":config.objectives,"decision_tick":state.decision_tick,"selection":state.objectives,"navigation":self.simulation.navigation[id],"memory":self.simulation.actors.inspect_state(self.simulation.definitions(),state)["memory"]}))
                }).collect();
                return Ok(json!({"tick":self.simulation.tick,"npcs":npcs}));
            }
            Command::InspectHearing => {
                let npcs:serde_json::Map<_,_>=self.simulation.npcs.iter().map(|(id,state)| {
                    let config=self.simulation.definitions().entities.iter().find(|e|e.id==*id).unwrap().npc.as_ref().unwrap();
                    (id.clone(),json!({"policy":config.sound_attention,"decision_tick":state.decision_tick,"attention":state.sound_attention,"heard_this_tick":state.heard_this_tick,"memory":self.simulation.actors.inspect_state(self.simulation.definitions(),state)["memory"]}))
                }).collect();
                return Ok(json!({"tick":self.simulation.tick,"npcs":npcs}));
            }
            Command::SetFactionRelations { relations } => {
                self.apply_edits(std::iter::once(SceneEdit::SetFactionRelations {
                    relations,
                }))?
            }
            Command::InspectNpcs => {
                return Ok(json!({"tick":self.simulation.tick,"npcs":self.npc_snapshot()}));
            }
            Command::SetNavigationGeneration { recipe } => {
                self.apply_edits(std::iter::once(SceneEdit::SetNavigationGeneration {
                    recipe,
                }))?;
            }
            Command::QueryNavigationGeneration { recipe } => {
                recipe.validate(&self.scene)?;
                let (network, report) = recipe.generate(&self.scene);
                return Ok(json!({"network":network,"report":report,"static_only":true}));
            }
            Command::SetSpatialNetwork { network } => {
                self.apply_edits(std::iter::once(SceneEdit::SetSpatialNetwork { network }))?
            }
            Command::InspectMobility => {
                return Ok(
                    json!({"tick":self.simulation.tick,"generation":self.simulation.definitions().navigation_generation.as_ref().map(|r|json!({"recipe":r,"report":r.cached.as_ref().map(|c|c.report)})),"network":self.simulation.definitions().spatial_network,"flight_profiles":self.simulation.definitions().entities.iter().filter_map(|e|e.npc.as_ref().filter(|n|n.mobility.is_some()).map(|n|(&e.id,n.flight.unwrap_or(crate::flight::Profile::Straight)))).collect::<std::collections::BTreeMap<_,_>>(),"local_connections":self.simulation.definitions().entities.iter().filter_map(|e|e.npc.as_ref().filter(|n|n.mobility.is_some()).map(|n|(&e.id,n.local_connections))).collect::<std::collections::BTreeMap<_,_>>(),"states":self.simulation.mobility.states,"compiled_links":self.simulation.mobility.compiled,"rejection_reasons":self.simulation.mobility.rejection_reasons,"ground_admissions":self.simulation.mobility.ground_reports.iter().map(|(id,r)|(id,r.as_ref())).collect::<std::collections::BTreeMap<_,_>>(),"policies":self.simulation.definitions().entities.iter().filter_map(|e|e.npc.as_ref().filter(|n|n.mobility.is_some()).map(|n|(&e.id,n.unreachable_policy.unwrap_or(crate::mobility::UnreachablePolicy::Stop)))).collect::<std::collections::BTreeMap<_,_>>()}),
                );
            }
            Command::SetPassages { passages } => {
                self.apply_edits(std::iter::once(SceneEdit::SetPassages { passages }))?
            }
            Command::InspectPassages => {
                let owners: Vec<_> = self
                    .simulation
                    .passages
                    .slots
                    .iter()
                    .map(|s| {
                        s.owner.map(|i| {
                            self.simulation.definitions().entities[i as usize]
                                .id
                                .as_str()
                        })
                    })
                    .collect();
                return Ok(
                    json!({"tick":self.simulation.tick,"policy":self.scene.passages,"runtime":self.simulation.passages,"owners":owners}),
                );
            }
            Command::InspectNavigation => {
                return Ok(json!({"tick":self.simulation.tick,"npcs":self.simulation.navigation}));
            }
            Command::Generate { recipe } => self.replace(recipe.generate()?)?,
            Command::PreviewSceneEdits { edits } => {
                let mut scene = self.staged_edits(edits.into_iter())?;
                scene.validate()?;
                scene.normalize();
                scene.entities.sort_by(|a, b| a.id.cmp(&b.id));
                return Ok(json!({"scene":scene}));
            }
            Command::EditScene { edits } => self.apply_edits(edits.into_iter())?,
            Command::EditSceneLive { edits, gesture } => {
                if gesture.is_empty() || gesture.len() > 128 || !gesture.is_ascii() {
                    return Err(invalid("Live edit gesture requires1..128 ASCII bytes"));
                }
                let mut scene = self.staged_edits(edits.into_iter())?;
                scene.validate()?;
                let coalesce = self
                    .live_edit
                    .as_ref()
                    .is_some_and(|(g, r)| *g == gesture && *r == self.revision);
                if coalesce {
                    scene.normalize();
                    scene.entities.sort_by(|a, b| a.id.cmp(&b.id));
                    self.scene = scene;
                    self.redo.clear();
                    self.revision += 1;
                    self.reset();
                } else {
                    self.replace(scene)?;
                }
                self.live_edit = Some((gesture, self.revision));
            }
            Command::Upsert { entity } => {
                self.apply_edits(std::iter::once(SceneEdit::Upsert { entity }))?
            }
            Command::Remove { id } => {
                self.apply_edits(std::iter::once(SceneEdit::Remove { id }))?
            }
            Command::SetCamera { camera } => {
                self.apply_edits(std::iter::once(SceneEdit::SetCamera { camera }))?
            }
            Command::Wind { .. } => unreachable!("wind command handled before dispatch"),
            Command::Particles { .. } => unreachable!("particle command handled before dispatch"),
            Command::SetLight { light } => {
                self.apply_edits(std::iter::once(SceneEdit::SetLight { light }))?
            }
            Command::Undo => {
                let previous = self
                    .undo
                    .pop_back()
                    .ok_or_else(|| invalid("Nothing to undo"))?;
                self.redo.push(std::mem::replace(&mut self.scene, previous));
                self.revision += 1;
                self.reset();
            }
            Command::Redo => {
                let next = self.redo.pop().ok_or_else(|| invalid("Nothing to redo"))?;
                self.undo
                    .push_back(std::mem::replace(&mut self.scene, next));
                self.revision += 1;
                self.reset();
            }
            Command::Play => {
                if !self.clock_open() {
                    return Err(invalid("Reset before starting a new session"));
                }
                self.playing = true;
            }
            Command::Pause => self.playing = false,
            Command::Reset => self.reset(),
            Command::SetInput { input } => {
                Self::validate_input(input)?;
                if self.simulation.player_count() > 1 || !self.controls.state.bindings.is_empty() {
                    return Err(invalid(
                        "Use an explicit control for multiple or bound players",
                    ));
                }
                self.input = input;
                self.input_revision += 1;
            }
            Command::Step { ticks } => {
                if self.playing {
                    return Err(invalid("Pause realtime playback before explicit stepping"));
                }
                self.advance(ticks)?;
            }
            Command::Events => {
                return Ok(
                    json!({"tick":self.simulation.tick,"events":self.simulation.events,"capacity":2048,"dropped_events":self.simulation.dropped_events}),
                );
            }
            Command::Exit => {
                self.exit_requested = true;
                self.playing = false;
            }
            _ => return Err(invalid("This command requires the project service")),
        }
        Ok(json!({"revision":self.revision,"tick":self.simulation.tick,"playing":self.playing}))
    }
}

fn default_normal_sample_limit() -> u8 {
    8
}

impl Engine {
    pub fn baseline_tick(&self) -> u64 {
        self.baseline_tick
    }
    pub(crate) fn capture_checkpoint(&self) -> Result<crate::checkpoint::Checkpoint> {
        if self.planet_active() {
            return Err(invalid(
                "Planet runtime primaries are not part of checkpoint3; refusing an incomplete save",
            ));
        }
        if self.playing
            || self.consumed_ability_inputs != self.ability_inputs.len()
            || self.consumed_population_inputs != self.population_inputs.len()
            || self.consumed_noises != self.noises.len()
        {
            return Err(invalid(
                "Pause and advance pending gameplay commands before checkpoint",
            ));
        }
        let mut manual = VecDeque::with_capacity(RECENT_MANUAL_NOISES);
        let mut dropped = self.dropped_manual_noises;
        for n in self.recent_manual_noises.iter().chain(&self.noises) {
            if self.simulation.tick.saturating_sub(n.tick) < 90 {
                if manual.len() == RECENT_MANUAL_NOISES {
                    manual.pop_front();
                    dropped = dropped.saturating_add(1);
                }
                manual.push_back(*n);
            }
        }
        Ok(crate::checkpoint::Checkpoint {
            version: crate::checkpoint::VERSION,
            scene_version: SCENE_VERSION,
            physics_version: "rapier3d-0.30.1".into(),
            platform: crate::checkpoint::platform(),
            build_id: env!("IGNIS_BUILD_ID").into(),
            scene: self.scene.clone(),
            mode: self.execution_mode,
            sequences: [
                self.admission.noise,
                self.admission.ability,
                self.admission.population,
            ],
            manual,
            dropped_manual: dropped,
            state: self.simulation.checkpoint(),
        })
    }
    pub fn from_checkpoint(bytes: &[u8], mode: Option<ExecutionMode>) -> Result<Self> {
        let saved = crate::checkpoint::Checkpoint::decode(bytes)?;
        let mut candidate =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<Self> {
                let mut candidate = Self::new(saved.scene.clone())?;
                candidate.simulation = Simulation::from_checkpoint(&candidate.scene, saved.state)?;
                candidate.controls =
                    crate::controls::Runtime::new(candidate.simulation.actors.actors.len());
                candidate.admission = Admission::new(candidate.simulation.actors.actors.len());
                candidate.baseline_tick = candidate.simulation.tick;
                candidate.baseline_sequences = saved.sequences;
                candidate.admission.noise = saved.sequences[0];
                candidate.admission.ability = saved.sequences[1];
                candidate.admission.population = saved.sequences[2];
                candidate.recent_manual_noises = saved.manual;
                candidate.dropped_manual_noises = saved.dropped_manual;
                candidate.execution_mode = mode.unwrap_or(saved.mode);
                candidate.input = Input::default();
                candidate.playing = false;
                Ok(candidate)
            }))
            .map_err(|_| invalid("Invalid checkpoint runtime structure"))??;
        candidate.baseline = Some(std::sync::Arc::from(bytes));
        Ok(candidate)
    }
    pub fn load_checkpoint(&mut self, bytes: &[u8], mode: Option<ExecutionMode>) -> Result<()> {
        if self.planet_active() {
            return Err(invalid(
                "Disable planet runtime before loading a checkpoint",
            ));
        }
        if self.playing {
            return Err(invalid("Pause before loading checkpoint"));
        }
        let mut candidate = Self::from_checkpoint(bytes, mode)?;
        candidate.debug_view = self.debug_view;
        candidate.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("Revision exhausted"))?;
        candidate.input_revision = self
            .input_revision
            .checked_add(1)
            .ok_or_else(|| invalid("Input revision exhausted"))?;
        candidate.view_revision = self
            .view_revision
            .checked_add(1)
            .ok_or_else(|| invalid("View revision exhausted"))?;
        *self = candidate;
        Ok(())
    }
}
