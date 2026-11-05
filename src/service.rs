mod volume_terrain;
use crate::{
    engine::{Command, Engine, Replay, Request},
    model::{Error, Result, Scene, invalid},
    render::{RenderFrame, Renderer},
};
use serde_json::{Value, json};
mod botany;
mod characters;
mod materials;
mod particles;
mod ui;
mod wind;
use std::{
    io::{BufReader, BufWriter, Read, Write},
    path::{Component, Path, PathBuf},
};

const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;

// Match the read boundary. Large mesh/pose documents use compact JSON when needed.
pub(crate) fn document_bytes(value: &impl serde::Serialize) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(io_error)?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        bytes.clear();
        serde_json::to_writer(&mut bytes, value).map_err(io_error)?;
    }
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(invalid(
            "Serialized document exceeds8 MiB; reduce content or recorded duration",
        ));
    }
    Ok(bytes)
}

pub struct Reply {
    pub data: Value,
    pub png: Option<Vec<u8>>,
}
pub struct Service {
    pub game_ui: crate::game_ui::State,
    pub wind_panel_open: bool,
    pub botany: crate::botany::State,
    pub particle_panel_open: bool,
    pub character_editor: crate::character_editor::State,
    pub engine: Engine,
    pub root: PathBuf,
    pub editor: crate::editor::State,
    pub selected_mesh: Option<String>,
    pub mesh_view: crate::editor::MeshView,
    pub transform: crate::editor_transform::Settings,
    pub scene_path: Option<String>,
    pub project: Option<crate::project::Manifest>,
    pub project_library: crate::project_browser::State,
    pub session_revision: u64,
    pub viewport_performance: crate::viewport_performance::State,
    pub(crate) saved_scene_revision: u64,
    editor_capture: Option<crate::editor_ui::Graphics>,
    capture_renderer: Option<Renderer>,
    volume_gpu: Option<crate::volume_terrain::gpu::Gpu>,
    planet_capture: Option<crate::planet::view::Gpu>,
    network_host: Option<crate::network::host::Host>,
    network_client: Option<crate::network::client::Native>,
    network_frame: Option<RenderFrame>,
    network_camera: Option<crate::model::Camera>,
    network_view_revision: u64,
    network_controls: crate::network::controls::Controls,
    audio_output: Option<crate::audio_device::Output>,
    audio_scope: String,
}
pub(crate) fn io_error(e: impl std::fmt::Display) -> Error {
    Error {
        code: "io_error",
        message: e.to_string(),
    }
}

// Atomic writes replace a filename, while parent directories may be aliases.
fn destination_key(dest: &Path) -> Result<PathBuf> {
    let mut existing = dest.parent().ok_or_else(|| invalid("Missing parent"))?;
    while !existing.exists() {
        existing = existing
            .parent()
            .ok_or_else(|| invalid("Invalid destination"))?;
    }
    Ok(existing
        .canonicalize()
        .map_err(io_error)?
        .join(dest.strip_prefix(existing).map_err(io_error)?))
}

impl Service {
    fn remote_state(
        &self,
    ) -> Result<std::sync::Arc<std::sync::Mutex<crate::network::client::State>>> {
        self.network_client
            .as_ref()
            .ok_or_else(|| invalid("No remote client"))?
            .state
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| invalid("Remote client is not ready"))
    }
    pub(crate) fn remote_input_context(&self) -> Option<crate::network::controls::Context> {
        let shared = self.remote_state().ok()?;
        crate::network::controls::Context::current(&shared.lock().unwrap())
    }
    pub(crate) fn remote_grant(&self, slot: u8) -> Option<crate::abilities::Selection> {
        let shared = self.remote_state().ok()?;
        shared
            .lock()
            .unwrap()
            .grants
            .iter()
            .find(|g| g.slot == slot)
            .map(|g| g.selection)
    }
    fn issue_remote(&mut self, operation: crate::network::protocol::Operation) -> Result<Value> {
        self.network_client
            .as_ref()
            .ok_or_else(|| invalid("No remote client"))?
            .operation(operation)
            .map_err(io_error)?;
        self.network_controls.issued(operation.id);
        Ok(json!({"submitted":true,"operation":operation.id,"committed":false}))
    }
    fn decorate_remote(&self, state: &crate::network::client::State, frame: &mut RenderFrame) {
        if let Some(target) = self.network_controls.selected
            && let Some(actor) = state.replica.actors().get(&target.actor)
            && actor.life == target.life
        {
            frame.selection = Some(crate::render::RenderBox {
                geometry: actor.appearance.geometry,
                position: actor.position(),
                size: actor.appearance.size,
                color: actor.appearance.color,
            });
        }
    }
    pub fn remote_client(&self) -> bool {
        self.network_client.is_some()
    }
    pub fn remote_stamp(&self) -> Option<(bool, u64, u64, u64, crate::model::MovementMode)> {
        self.network_client.as_ref().map(|client| {
            let shared = client.state.lock().unwrap();
            if let Some(state) = shared.as_ref() {
                let state = state.lock().unwrap();
                (
                    state.connected,
                    state.replica.tick().unwrap_or(0),
                    state.world,
                    self.network_view_revision,
                    self.network_controls.mode(),
                )
            } else {
                (false, 0, 0, self.network_view_revision, Default::default())
            }
        })
    }
    pub fn update_game_frame(&self, frame: &mut RenderFrame) {
        if let Ok(shared) = self.remote_state() {
            let state = shared.lock().unwrap();
            if crate::network::presentation::update(&state, frame, self.network_camera.as_ref())
                .is_ok()
            {
                self.decorate_remote(&state, frame);
                return;
            }
        }
        if self.remote_client() {
            *frame = crate::network::presentation::empty(
                self.engine.scene.camera.clone(),
                self.engine.scene.light.clone(),
            );
        } else if self.botany.revision == Some(self.engine.revision)
            && let Some(overlay) = &self.botany.overlay
        {
            self.engine.update_render_frame_with_meshes(frame, overlay);
        } else {
            self.engine.update_render_frame(frame);
        }
    }
    pub fn game_frame(&self) -> RenderFrame {
        let mut frame = crate::network::presentation::empty(
            self.engine.scene.camera.clone(),
            self.engine.scene.light.clone(),
        );
        self.update_game_frame(&mut frame);
        frame
    }
    pub fn poll_network(&mut self) -> Result<()> {
        if let Some(host) = &mut self.network_host {
            host.poll(&mut self.engine)?;
        }
        if let Ok(shared) = self.remote_state() {
            self.network_controls.sync(&shared.lock().unwrap());
        }
        Ok(())
    }
    /// All native/headless clocks cross the same optional authority boundary.
    pub fn advance(&mut self, ticks: u32) -> Result<()> {
        if self.network_client.is_some() {
            return Err(invalid("Remote client does not simulate the local world"));
        }
        if self.network_host.is_none() && self.audio_output.is_none() {
            return self.advance_botany(ticks);
        }
        if !(1..=3600).contains(&ticks)
            || self
                .engine
                .simulation
                .tick
                .checked_add(ticks as u64)
                .is_none_or(|t| t > crate::engine::MAX_RUNTIME_TICK)
        {
            return Err(invalid("Step requires1..3600ticks and available clock"));
        }
        for _ in 0..ticks {
            if let Some(host) = self.network_host.as_mut() {
                if let Err(error) = host
                    .before_tick(&mut self.engine)
                    .and_then(|_| self.engine.advance(1))
                    .and_then(|_| host.after_tick(&mut self.engine))
                {
                    host.close(&mut self.engine);
                    return Err(error);
                }
            } else {
                self.advance_botany(1)?;
            }
            self.feed_audio()?;
        }
        Ok(())
    }
    fn sync_audio_scope(&mut self) {
        if self.audio_scope != self.engine.runtime_scope() {
            if let Some(output) = &self.audio_output {
                output.invalidate();
            }
            self.audio_scope = self.engine.runtime_scope().to_owned();
        }
    }
    fn feed_audio(&mut self) -> Result<()> {
        if self.audio_output.is_none() {
            return Ok(());
        }
        self.sync_audio_scope();
        let output = self.audio_output.as_ref().unwrap();
        let samples = crate::npc_presentation::pcm(
            &self.engine.simulation.definitions().avatar_audio,
            &self.engine.simulation.presentation.voices,
            self.engine.simulation.tick,
            400,
            output.listener,
            output.radius,
        )?;
        output.submit(samples);
        Ok(())
    }
    pub fn new(root: &Path, scene: Scene) -> Result<Self> {
        let root = root.canonicalize().map_err(io_error)?;
        if !root.is_dir() {
            return Err(invalid("Project root must be a directory"));
        }
        Ok(Self {
            wind_panel_open: false,
            character_editor: Default::default(),
            botany: Default::default(),
            particle_panel_open: false,
            engine: Engine::new(scene)?,
            root,
            capture_renderer: None,
            volume_gpu: None,
            planet_capture: None,
            editor: crate::editor::State::default(),
            selected_mesh: None,
            mesh_view: Default::default(),
            transform: Default::default(),
            scene_path: None,
            project: None,
            project_library: Default::default(),
            session_revision: 0,
            saved_scene_revision: 0,
            editor_capture: None,
            viewport_performance: Default::default(),
            game_ui: Default::default(),
            network_host: None,
            network_client: None,
            network_frame: None,
            network_camera: None,
            network_view_revision: 0,
            network_controls: Default::default(),
            audio_output: None,
            audio_scope: String::new(),
        })
    }
    pub fn select_mesh(&mut self, instance: Option<String>) -> Result<Value> {
        if instance.as_ref().is_some_and(|id| {
            !self
                .engine
                .scene
                .meshes
                .instances
                .iter()
                .any(|i| &i.id == id)
        }) {
            return Err(invalid("Mesh instance not found"));
        }
        self.engine
            .execute(Command::SelectEntity { id: None }.into())?;
        if self.selected_mesh != instance {
            self.selected_mesh = instance;
            self.session_revision = self.session_revision.wrapping_add(1);
        }
        Ok(json!({"selected_mesh":self.selected_mesh}))
    }
    pub fn pick_mesh(
        &self,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
    ) -> Result<Option<crate::mesh_tools::Hit>> {
        crate::mesh_tools::pick(
            &self.engine.scene.meshes,
            self.editor
                .current()
                .camera
                .as_ref()
                .unwrap_or(&self.engine.scene.camera),
            x,
            y,
            width,
            height,
            self.engine.simulation.tick,
            &self
                .engine
                .scene
                .plants
                .connections
                .iter()
                .map(|c| c.child_instance.as_str())
                .collect::<Vec<_>>(),
        )
    }
    /// Resolve inside a designated project root. Reject traversal and symlink escapes.
    pub fn resolve(&self, path: &str) -> Result<PathBuf> {
        let relative = Path::new(path);
        if path.is_empty()
            || path.len() > 512
            || relative
                .components()
                .any(|p| !matches!(p, Component::Normal(_)))
        {
            return Err(invalid(
                "Use a relative project path without . or .. components",
            ));
        }
        let dest = self.root.join(relative);
        let mut existing = dest.as_path();
        while !existing.exists() {
            if existing.symlink_metadata().is_ok() {
                return Err(invalid("Dangling symlinks are not allowed"));
            }
            existing = existing.parent().ok_or_else(|| invalid("Invalid path"))?;
        }
        if !existing
            .canonicalize()
            .map_err(io_error)?
            .starts_with(&self.root)
        {
            return Err(invalid("Path escapes project root through a symlink"));
        }
        Ok(dest)
    }
    pub(crate) fn read<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let file = std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
        let mut bytes = Vec::new();
        file.take((MAX_DOCUMENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() > MAX_DOCUMENT_BYTES {
            return Err(invalid("File exceeds 8 MiB"));
        }
        serde_json::from_slice(&bytes).map_err(|e| invalid(format!("Invalid document: {e}")))
    }
    // Local authoring scenes scale with their content. Keep bounded envelopes for
    // replays, editor layouts and other documents; never allocate a second full
    // scene-sized byte buffer just to parse this file. Engine validates the whole
    // candidate before replacing the current world.
    pub(crate) fn read_scene(&self, path: &str) -> Result<Scene> {
        let source = self.resolve(path)?;
        if !std::fs::metadata(&source).map_err(io_error)?.is_file() {
            return Err(invalid("Scene must be a regular local file"));
        }
        let file = std::fs::File::open(source).map_err(io_error)?;
        if !file.metadata().map_err(io_error)?.is_file() {
            return Err(invalid("Scene must be a regular local file"));
        }
        serde_json::from_reader(BufReader::new(file))
            .map_err(|e| invalid(format!("Invalid scene document: {e}")))
    }
    fn write_scene(&self, path: &str) -> Result<PathBuf> {
        let dest = self.resolve(path)?;
        let parent = dest
            .parent()
            .ok_or_else(|| invalid("Missing parent directory"))?;
        std::fs::create_dir_all(parent).map_err(io_error)?;
        self.resolve(path)?;
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
        {
            let mut writer = BufWriter::new(temp.as_file_mut());
            serde_json::to_writer(&mut writer, &self.engine.scene).map_err(io_error)?;
            writer.flush().map_err(io_error)?;
        }
        temp.as_file().sync_all().map_err(io_error)?;
        temp.persist(&dest).map_err(io_error)?;
        Ok(dest)
    }
    pub(crate) fn write(&self, path: &str, bytes: &[u8]) -> Result<PathBuf> {
        let dest = self.resolve(path)?;
        let parent = dest
            .parent()
            .ok_or_else(|| invalid("Missing parent directory"))?;
        std::fs::create_dir_all(parent).map_err(io_error)?;
        // Recheck after creating parents, before writing.
        self.resolve(path)?;
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(io_error)?;
        temp.write_all(bytes).map_err(io_error)?;
        temp.as_file().sync_all().map_err(io_error)?;
        temp.persist(&dest).map_err(io_error)?;
        Ok(dest)
    }
    pub fn execute(&mut self, request: Request) -> Result<Reply> {
        self.engine.guard(request.expected_revision)?;
        if self.engine.inspection_flight_active()
            && matches!(
                &request.command,
                Command::SaveCheckpoint { .. }
                    | Command::SavePortableSave { .. }
                    | Command::StartMultiplayerHost { .. }
                    | Command::ConnectMultiplayer { .. }
                    | Command::SetPlanetRuntime { .. }
                    | Command::LoadPlanetRuntime { .. }
            )
        {
            return Err(invalid(
                "Disable inspection flight before persistence, networking or planet runtime",
            ));
        }
        if self.remote_client()
            && matches!(
                &request.command,
                Command::SetInspectionFlight { .. } | Command::SetInspectionFlightVertical { .. }
            )
        {
            return Err(invalid("Inspection flight is offline only"));
        }

        if matches!(
            &request.command,
            Command::SetPlanetRuntime { .. } | Command::LoadPlanetRuntime { .. }
        ) && (self.network_host.is_some() || self.network_client.is_some())
        {
            return Err(invalid(
                "Planet runtime adapter is offline only; stop multiplayer first",
            ));
        }
        if self.engine.planet_active()
            && matches!(
                &request.command,
                Command::SaveScene { .. }
                    | Command::StartMultiplayerHost { .. }
                    | Command::ConnectMultiplayer { .. }
                    | Command::LoadCheckpoint { .. }
                    | Command::LoadPortableSave { .. }
                    | Command::EditEditor { .. }
                    | Command::CaptureEditor { .. }
                    | Command::LoadEditor { .. }
                    | Command::SaveEditor { .. }
                    | Command::CreateProject { .. }
                    | Command::OpenProject { .. }
                    | Command::NewProjectScene { .. }
                    | Command::OpenProjectScene { .. }
                    | Command::SetProjectEntry { .. }
            )
        {
            return Err(invalid(
                "This operation has not migrated planet runtime primaries; disable planet runtime explicitly first",
            ));
        }
        let clear_mesh = matches!(
            &request.command,
            Command::SelectEntity { .. }
                | Command::LoadScene { .. }
                | Command::LoadCheckpoint { .. }
                | Command::LoadPortableSave { .. }
                | Command::CreateProject { .. }
                | Command::OpenProject { .. }
                | Command::NewProjectScene { .. }
                | Command::OpenProjectScene { .. }
        );
        if matches!(
            &request.command,
            Command::Botany { .. } | Command::OpenCharacterPanel { .. } | Command::Particles { .. }
        ) {
            self.wind_panel_open = false;
        }
        let clear_botany = (clear_mesh
            && !matches!(&request.command, Command::SelectEntity { .. }))
            || matches!(
                &request.command,
                Command::StartMultiplayerHost { .. } | Command::ConnectMultiplayer { .. }
            );
        let session_change = matches!(
            &request.command,
            Command::Wind { .. }
                | Command::Particles { .. }
                | Command::Botany { .. }
                | Command::OpenCharacterPanel { .. }
                | Command::SetCharacterHandleView { .. }
                | Command::LoadCharacterEditorSource { .. }
                | Command::CreateProject { .. }
                | Command::OpenProject { .. }
                | Command::NewProjectScene { .. }
                | Command::OpenProjectScene { .. }
                | Command::SetProjectEntry { .. }
                | Command::SaveScene { .. }
                | Command::LoadScene { .. }
                | Command::LoadCheckpoint { .. }
                | Command::LoadPortableSave { .. }
        );
        if matches!(
            &request.command,
            Command::OpenCharacterPanel { open: true, .. }
        ) {
            self.botany.open = false;
            self.particle_panel_open = false;
        }
        if matches!(&request.command,Command::Botany{operation} if matches!(operation.as_ref(),crate::botany::Operation::Open{open:true,..}))
        {
            self.particle_panel_open = false;
        }
        if clear_botany {
            self.particle_panel_open = false;
        }
        let pause_audio = matches!(&request.command, Command::Pause | Command::Exit);
        let data = match &request.command {
            Command::Ui {
                operation,
                expected_ui_revision,
            } => self.ui_command(operation, *expected_ui_revision)?,
            Command::Particles { operation } => self.particles_command(operation)?,
            Command::Material { operation } => self.material_command(operation)?,
            Command::Wind { operation } => self.wind_command(operation)?,
            Command::Botany { operation } => self.botany_command(operation)?,
            Command::InspectCharacterSource { .. }
            | Command::PublishCharacter { .. }
            | Command::ConnectCharacterSource { .. }
            | Command::UpgradeCharacterSource { .. }
            | Command::OpenCharacterPanel { .. }
            | Command::InspectCharacterEditor
            | Command::SetCharacterHandleView { .. }
            | Command::PreviewCharacterHandle { .. }
            | Command::SetCharacterHandle { .. }
            | Command::LoadCharacterEditorSource { .. }
            | Command::SetCharacterParameters { .. }
            | Command::RegenerateCharacter { .. }
            | Command::SaveCharacterConfiguration { .. }
            | Command::LoadCharacterConfiguration { .. } => self.character_command(&request.command)?,
            Command::SaveLandform {path,document} => {
                if !path.ends_with(".ignis-landform") {return Err(invalid("Landform path requires .ignis-landform suffix"));}
                let bytes=crate::landforms::encode(document)?;
                let destination=self.write(path,&bytes)?;
                json!({"path":destination,"bytes":bytes.len(),"format":document.version,"id":document.id,"revision":document.revision,"world_changed":false})
            }
            Command::LoadLandform {path} => {
                if !path.ends_with(".ignis-landform") {return Err(invalid("Landform path requires .ignis-landform suffix"));}
                let file=std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
                let mut bytes=Vec::new();file.take((crate::terrain::MAX_BYTES+1) as u64).read_to_end(&mut bytes).map_err(io_error)?;
                let document=crate::landforms::decode(&bytes)?;
                json!({"document":document,"bytes":bytes.len(),"world_changed":false})
            }
            Command::CapturePlanetReference { document, view, path, width, height } => {
                self.validate_capture(path, *width, *height)?;
                let prepared = view.prepare(document, *width, *height)?;
                if self.capture_renderer.is_none() {
                    let instance = wgpu::Instance::new(&Default::default());
                    self.capture_renderer = Some(pollster::block_on(Renderer::new(&instance,None)).map_err(|e| Error{code:"graphics_unavailable",message:e.to_string()})?);
                }
                let renderer = self.capture_renderer.as_ref().unwrap();
                let gpu = self.planet_capture.get_or_insert_with(||crate::planet::view::Gpu::new(renderer));
                let bytes = gpu.capture(renderer,&prepared).map_err(|e|Error{code:"capture_failed",message:e.to_string()})?;
                let mut data = prepared.metadata();
                data["build_id"]=json!(env!("IGNIS_BUILD_ID"));
                data["source_id"]=json!(document.id);
                data["diameter_meters"]=json!(document.diameter_meters);
                data["view"]=json!(view);
                data["mime_type"]=json!("image/png");
                data["width"]=json!(width);
                data["height"]=json!(height);
                data["adapter"]=json!(renderer.info());
                data["path"]=json!(self.write(path,&bytes)?);
                return Ok(Reply{data,png:Some(bytes)});
            }
            Command::SavePlanet { path, document } => {
                if !path.ends_with(".ignis-planet") { return Err(invalid("Planet path requires .ignis-planet suffix")); }
                let bytes = crate::planet::encode(document)?;
                let destination = self.write(path, &bytes)?;
                json!({"path":destination,"bytes":bytes.len(),"format":document.version,"world_changed":false})
            }
            Command::LoadPlanet { path } => {
                if !path.ends_with(".ignis-planet") { return Err(invalid("Planet path requires .ignis-planet suffix")); }
                let file = std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
                let mut bytes=Vec::new();
                file.take((crate::terrain::MAX_BYTES+1) as u64).read_to_end(&mut bytes).map_err(io_error)?;
                let document = crate::planet::decode(&bytes)?;
                json!({"document":document,"bytes":bytes.len(),"world_changed":false})
            }
            Command::QueryVolumeTerrainGpu{..}|Command::PublishVolumeTerrainGpu{..}=>self.volume_terrain_gpu_command(&request.command)?,
            Command::SaveVolumeTerrain {path,document} => {
                if !path.ends_with(".ignis-volume"){return Err(invalid("Density path requires .ignis-volume suffix"));}
                let bytes=crate::volume_terrain::encode(document)?;
                json!({"path":self.write(path,&bytes)?,"bytes":bytes.len(),"world_changed":false})
            }
            Command::LoadVolumeTerrain {path} => {
                if !path.ends_with(".ignis-volume"){return Err(invalid("Density path requires .ignis-volume suffix"));}
                let file=std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
                let mut bytes=Vec::new();file.take((crate::volume_terrain::MAX_BYTES+1)as u64).read_to_end(&mut bytes).map_err(io_error)?;
                json!({"document":crate::volume_terrain::decode(&bytes)?,"bytes":bytes.len(),"world_changed":false})
            }
            Command::SaveTerrain { path, document } => {
                if !path.ends_with(".ignis-terrain") {return Err(invalid("Terrain path requires .ignis-terrain suffix"));}
                let bytes=crate::terrain::encode(document)?;
                let destination=self.write(path,&bytes)?;
                json!({"path":destination,"bytes":bytes.len(),"format":document.version,"id":document.id,"revision":document.revision,"world_changed":false})
            }
            Command::LoadTerrain { path } => {
                if !path.ends_with(".ignis-terrain") {return Err(invalid("Terrain path requires .ignis-terrain suffix"));}
                let file=std::fs::File::open(self.resolve(path)?).map_err(io_error)?;
                let mut bytes=Vec::new(); file.take((crate::terrain::MAX_BYTES+1) as u64).read_to_end(&mut bytes).map_err(io_error)?;
                let document=crate::terrain::decode(&bytes)?;
                json!({"document":document,"bytes":bytes.len(),"world_changed":false})
            }
            Command::InspectNpcPresentation if self.remote_client() => {
                let shared=self.remote_state()?;
                let state=shared.lock().unwrap();
                json!({"tick":state.replica.tick(),"public_only":true,"poses":state.replica.avatars(),"audio":{"device_output":false,"bank_available":false}})
            }
            Command::InspectNpcPresentation => {
                let mut value=self.engine.execute(Command::InspectNpcPresentation.into())?;
                value["audio"]["device_output"]=json!(self.audio_output.is_some());
                value["audio"]["output"]=self.audio_output.as_ref().map_or_else(||json!({"enabled":false}), |o|o.inspect());
                value
            }
            Command::SetNpcAudioOutput { enabled, listener, radius } => {
                if *enabled {
                    if self.remote_client() { return Err(invalid("Remote content does not carry the local PCM bank")); }
                    let output = crate::audio_device::Output::start(*listener, *radius)?;
                    self.audio_output = Some(output);
                    self.audio_scope = self.engine.runtime_scope().to_owned();
                } else { self.audio_output = None; }
                self.audio_output.as_ref().map_or_else(||json!({"enabled":false}), |o|o.inspect())
            }
            Command::InspectNpcAudioOutput => self.audio_output.as_ref().map_or_else(||json!({"enabled":false,"compiled":cfg!(feature="device-audio"),"source_rate":crate::npc_presentation::RATE}), |o|o.inspect()),
            Command::SetMultiplayerInput { input } => {
                let shared = self.remote_state()?;
                let (token, hold, revision) = {
                    let state = shared.lock().unwrap();
                    self.network_controls.sync(&state);
                    self.network_controls.prepare_hold(&state, *input)?
                };
                self.network_client
                    .as_ref()
                    .unwrap()
                    .hold(hold)
                    .map_err(io_error)?;
                self.network_controls
                    .adopt_hold(token.clone(), hold, revision);
                json!({"control":token,"input":input,"heartbeat_ms":100})
            }
            Command::ReleaseMultiplayerInput { control } => {
                let shared = self.remote_state()?;
                let neutral = self
                    .network_controls
                    .release(&shared.lock().unwrap(), control);
                if let Some(hold) = neutral {
                    self.network_client
                        .as_ref()
                        .unwrap()
                        .hold(hold)
                        .map_err(io_error)?;
                }
                json!({"released":neutral.is_some()})
            }
            Command::InspectMultiplayerControls => {
                let shared = self.remote_state()?;
                self.network_controls.sync(&shared.lock().unwrap());
                self.network_controls.inspect()
            }
            Command::SelectMultiplayerTarget { target } => {
                let shared = self.remote_state()?;
                let state = shared.lock().unwrap();
                if let Some(target) = target {
                    crate::network::controls::Controls::target(&state, *target)?;
                }
                let revision = self
                    .network_view_revision
                    .checked_add(1)
                    .ok_or_else(|| invalid("Remote view revision exhausted"))?;
                self.network_controls.sync(&state);
                self.network_controls.selected = *target;
                self.network_view_revision = revision;
                json!({"target":target})
            }
            Command::PickMultiplayer {
                x,
                y,
                width,
                height,
            } => {
                let shared = self.remote_state()?;
                let state = shared.lock().unwrap();
                let camera = self
                    .network_camera
                    .as_ref()
                    .or_else(|| state.content.as_ref().map(|c| &c.camera))
                    .ok_or_else(|| invalid("Remote camera is not ready"))?;
                let target =
                    crate::network::presentation::pick(&state, camera, *x, *y, *width, *height)?;
                let revision = self
                    .network_view_revision
                    .checked_add(1)
                    .ok_or_else(|| invalid("Remote view revision exhausted"))?;
                self.network_controls.sync(&state);
                self.network_controls.selected = target;
                self.network_view_revision = revision;
                json!({"target":target})
            }
            Command::UseMultiplayerAbility { slot, target, aim } => {
                let shared = self.remote_state()?;
                let operation = self.network_controls.use_ability(
                    &shared.lock().unwrap(),
                    *slot,
                    *target,
                    *aim,
                )?;
                self.issue_remote(operation)?
            }
            Command::UseMultiplayerAbilityAt {
                slot,
                x,
                y,
                width,
                height,
                plane_y,
            } => {
                let shared = self.remote_state()?;
                let state = shared.lock().unwrap();
                let camera = self
                    .network_camera
                    .as_ref()
                    .or_else(|| state.content.as_ref().map(|c| &c.camera))
                    .ok_or_else(|| invalid("Remote camera is not ready"))?;
                let plane = plane_y.unwrap_or(
                    state
                        .owner
                        .as_ref()
                        .ok_or_else(|| invalid("Remote owner not ready"))?
                        .position[1],
                );
                let position =
                    crate::picking::point_on_plane(camera, *x, *y, *width, *height, plane)?;
                let operation = self.network_controls.use_ability(
                    &state,
                    *slot,
                    None,
                    Some(crate::abilities::Aim::Point { position }),
                )?;
                drop(state);
                self.issue_remote(operation)?
            }
            Command::CancelMultiplayerAbility => {
                let shared = self.remote_state()?;
                let state = shared.lock().unwrap();
                let execution = state
                    .owner
                    .as_ref()
                    .and_then(|o| o.cast_execution)
                    .ok_or_else(|| invalid("No active remote cast"))?;
                let operation = self.network_controls.prepare_operation(
                    &state,
                    crate::network::protocol::Intent::Cancel { execution },
                )?;
                drop(state);
                self.issue_remote(operation)?
            }
            Command::SetMultiplayerView { camera } => {
                if !self.remote_client() {
                    return Err(invalid("No remote client view"));
                }
                camera.validate()?;
                let next = self
                    .network_view_revision
                    .checked_add(1)
                    .ok_or_else(|| invalid("Remote view revision exhausted"))?;
                self.network_camera = Some(camera.clone());
                self.network_view_revision = next;
                json!({"camera":camera,"view_revision":next})
            }
            Command::CaptureMultiplayer {
                path,
                width,
                height,
            } => {
                self.validate_capture(path, *width, *height)?;
                let shared = self
                    .network_client
                    .as_ref()
                    .ok_or_else(|| invalid("No remote client"))?
                    .state
                    .lock()
                    .unwrap()
                    .clone()
                    .ok_or_else(|| invalid("Remote client not ready"))?;
                let state = shared.lock().unwrap();
                let mut frame = self.network_frame.take().unwrap_or_else(|| {
                    crate::network::presentation::empty(
                        self.engine.scene.camera.clone(),
                        self.engine.scene.light.clone(),
                    )
                });
                if let Err(error) = crate::network::presentation::update(
                    &state,
                    &mut frame,
                    self.network_camera.as_ref(),
                ) {
                    self.network_frame = Some(frame);
                    return Err(error);
                }
                self.decorate_remote(&state, &mut frame);
                let (world, tick) = (state.world, state.replica.tick());
                drop(state);
                let result = self.capture_frame(&frame, path, *width, *height);
                self.network_frame = Some(frame);
                let mut reply = result?;
                reply.data["world"] = json!(world);
                reply.data["tick"] = json!(tick);
                reply.data["presentation"] = json!("remote");
                return Ok(reply);
            }
            Command::StartMultiplayerHost { address, capacity } => {
                if !self.engine.scene.wind.is_empty() {
                    return Err(invalid("Wind/free aerodynamic bodies are not yet replicated"));
                }
                if self.network_client.is_some() {
                    return Err(invalid("Disconnect the remote client before hosting"));
                }
                if self.network_host.is_some() {
                    return Err(invalid("Stop the current host first"));
                }
                let address = address
                    .parse()
                    .map_err(|_| invalid("Expected numeric host IP:port"))?;
                self.network_host = Some(crate::network::host::Host::launch(
                    &mut self.engine,
                    address,
                    *capacity,
                )?);
                self.engine.playing = true;
                self.network_host.as_ref().unwrap().inspect()
            }
            Command::StopMultiplayerHost => {
                if let Some(mut host) = self.network_host.take() {
                    host.close(&mut self.engine);
                }
                json!({"state":"stopped"})
            }
            Command::InspectMultiplayerHost => self
                .network_host
                .as_ref()
                .map_or(json!({"state":"offline"}), |h| h.inspect()),
            Command::ConnectMultiplayer {
                address,
                certificate_der,
            } => {
                if self.network_host.is_some() {
                    return Err(invalid(
                        "Stop the host before connecting this process as a remote client",
                    ));
                }
                use base64::Engine as _;
                if self.network_client.is_some() {
                    return Err(invalid("Disconnect the current client first"));
                }
                if certificate_der.len() > 24 * 1024 {
                    return Err(invalid("Certificate exceeds budget"));
                }
                let certificate = base64::engine::general_purpose::STANDARD
                    .decode(certificate_der)
                    .map_err(io_error)?;
                let address = address
                    .parse()
                    .map_err(|_| invalid("Expected numeric server IP:port"))?;
                self.network_client = Some(
                    crate::network::client::Native::launch(address, certificate, None)
                        .map_err(io_error)?,
                );
                self.engine.playing = false;
                self.network_frame = None;
                self.network_camera = None;
                json!({"state":"connecting"})
            }
            Command::ReconnectMultiplayer => {
                let next = self
                    .network_client
                    .as_ref()
                    .ok_or_else(|| invalid("No client session to resume"))?
                    .reconnect()
                    .map_err(io_error)?;
                self.network_client = Some(next);
                json!({"state":"connecting"})
            }
            Command::DisconnectMultiplayer => {
                self.network_client = None;
                self.network_frame = None;
                self.network_camera = None;
                json!({"state":"stopped"})
            }
            Command::InspectMultiplayerClient { actors } => self
                .network_client
                .as_ref()
                .map_or(json!({"state":"offline"}), |c| c.inspect(*actors)),
            Command::SendMultiplayerMovement { movement } => {
                Engine::validate_input(movement.input)?;
                self.network_client
                    .as_ref()
                    .ok_or_else(|| invalid("No multiplayer client"))?
                    .movement(*movement)
                    .map_err(io_error)?;
                self.network_controls.clear_input();
                json!({"submitted":true,"sequence":movement.sequence})
            }
            Command::SendMultiplayerOperation { operation } => {
                self.network_client
                    .as_ref()
                    .ok_or_else(|| invalid("No multiplayer client"))?
                    .operation(*operation)
                    .map_err(io_error)?;
                if let Ok(shared) = self.remote_state() {
                    let state = shared.lock().unwrap();
                    self.network_controls.sync(&state);
                    if state.last_operation.checked_add(1) == Some(operation.id)
                        && operation.life == state.actor.life
                    {
                        self.network_controls.issued(operation.id);
                    }
                }
                json!({"submitted":true,"operation":operation.id,"committed":false})
            }
            Command::Step { .. } | Command::Play if self.remote_client() => {
                return Err(invalid("Remote client does not simulate the local world"));
            }
            Command::Step { ticks }
                if self.network_host.is_some()
                    || self.audio_output.is_some()
                    || self.botany.physics.as_ref().is_some_and(|p| p.playing) =>
            {
                if self.engine.playing {
                    return Err(invalid("Pause realtime playback before explicit stepping"));
                }
                self.advance(*ticks)?;
                self.engine.execute(Command::Inspect.into())?
            }
            Command::ModelEditorMesh {
                instance,
                reference,
                operation,
                preview,
            } => crate::editor_modeling::execute(self, instance, reference, operation, *preview)?,
            Command::SetEditorTransform { settings } => {
                settings.validate()?;
                if self.transform != *settings {
                    self.transform = settings.clone();
                    self.session_revision += 1;
                }
                json!({"transform":self.transform})
            }
            Command::InspectEditorTransform { components } => {
                json!({"settings":self.transform,"selection":crate::editor_transform::selection(self,*components)?})
            }
            Command::PreviewEditorTransform {
                target,
                reference,
                transform,
            } => {
                json!({"scene":crate::editor_transform::preview(self,target,reference,transform)?})
            }
            Command::TransformEditor {
                target,
                reference,
                transform,
            } => crate::editor_transform::commit(self, target, reference, transform)?,
            Command::CreateEditorMesh {
                primitive,
                resolution,
                size,
            } => {
                if self.engine.playing {
                    return Err(invalid("Pause simulation before creating a mesh"));
                }
                let geometry = crate::mesh_tools::primitive(*primitive, *resolution, *size)?;
                let n = (0..1024)
                    .find(|n| {
                        let id = format!("mesh_{n}");
                        !self.engine.scene.meshes.assets.iter().any(|a| a.id == id)
                            && !self
                                .engine
                                .scene
                                .meshes
                                .instances
                                .iter()
                                .any(|i| i.id == id)
                    })
                    .ok_or_else(|| invalid("Mesh names exhausted"))?;
                let id = format!("mesh_{n}");
                let asset = crate::meshes::Asset {
                    id: id.clone(),
                    geometry: std::sync::Arc::new(geometry),
                    shading: crate::meshes::Shading::Smooth,
                    material: Default::default(),
                    rig: None,
                    clips: vec![],
                };
                let instance = crate::meshes::Instance {
                    id: id.clone(),
                    asset: id.clone(),
                    position: [2., size * 0.5 + 0.05, 0.],
                    rotation: [0., 0., 0., 1.],
                    scale: 1.,
                    color: [0.68, 0.72, 0.78],
                    playback: None,
                    attachment: None,
                };
                self.engine.execute(
                    Command::EditScene {
                        edits: vec![
                            crate::engine::SceneEdit::Mesh {
                                edit: crate::meshes::Edit::PutAsset { asset },
                            },
                            crate::engine::SceneEdit::Mesh {
                                edit: crate::meshes::Edit::PutInstance { instance },
                            },
                        ],
                    }
                    .into(),
                )?;
                self.select_mesh(Some(id))?
            }
            Command::SelectEditorMesh { instance } => self.select_mesh(instance.clone())?,
            Command::PickEditorMesh {
                x,
                y,
                width,
                height,
            } => json!({"hit":self.pick_mesh(*x,*y,*width,*height)?}),
            Command::InspectProject => self.project_status(),
            Command::BrowseProjectDirectory { query } => {
                json!({"directory":crate::project_browser::browse(query)?})
            }
            Command::CreateProjectFolder { parent, name } => {
                json!({"created_folder":crate::project_browser::create_folder(parent,name)?})
            }
            Command::InspectProjectLibrary => self.project_library.inspect(),
            Command::UseProjectLibrary { path } => {
                self.project_library.load(path)?;
                self.session_revision = self.session_revision.wrapping_add(1);
                self.project_library.inspect()
            }
            Command::RegisterProject { path } => {
                self.project_library.register(path)?;
                self.session_revision = self.session_revision.wrapping_add(1);
                self.project_library.inspect()
            }
            Command::PinProject { path, pinned } => {
                self.project_library.pin(path, *pinned)?;
                self.session_revision = self.session_revision.wrapping_add(1);
                self.project_library.inspect()
            }
            Command::ForgetProject { path } => {
                self.project_library.forget(path)?;
                self.session_revision = self.session_revision.wrapping_add(1);
                self.project_library.inspect()
            }
            Command::SetProjectFavorite { path, favorite } => {
                self.project_library.favorite(path, *favorite)?;
                self.session_revision = self.session_revision.wrapping_add(1);
                self.project_library.inspect()
            }
            Command::DiscoverProjects { paths } => {
                let result = self.project_library.discover(paths)?;
                self.session_revision = self.session_revision.wrapping_add(1);
                result
            }
            Command::CreateProject {
                path,
                name,
                discard_changes,
            } => self.create_project(path, name, *discard_changes)?,
            Command::OpenProject {
                path,
                discard_changes,
            } => self.open_project(path, *discard_changes)?,
            Command::NewProjectScene {
                name,
                discard_changes,
            } => self.new_project_scene(name, *discard_changes)?,
            Command::OpenProjectScene {
                path,
                discard_changes,
            } => self.open_project_scene(path, *discard_changes)?,
            Command::SetProjectEntry { path } => self.set_project_entry(path)?,
            Command::CreateSceneObject { kind } => self.create_scene_object(*kind, None)?,
            Command::DuplicateSceneObject { id } => {
                self.create_scene_object(crate::project::ObjectKind::Visual, Some(id))?
            }
            Command::SelectEditorAt {
                x,
                y,
                width,
                height,
            } => {
                let camera = self
                    .editor
                    .current()
                    .camera
                    .as_ref()
                    .unwrap_or(&self.engine.scene.camera);
                let hit = crate::picking::pick_camera(
                    &self.engine.simulation,
                    camera,
                    *x,
                    *y,
                    *width,
                    *height,
                )?;
                let mesh_hit = self.pick_mesh(*x, *y, *width, *height)?;
                if let Some(mesh) = mesh_hit
                    && hit.is_none_or(|actor| mesh.distance < actor.distance)
                {
                    let data = self.select_mesh(Some(mesh.instance))?;
                    return Ok(Reply { data, png: None });
                }
                self.selected_mesh = None;
                let id = hit.map(|h| {
                    self.engine.simulation.definitions().entities[h.entity_index as usize]
                        .id
                        .clone()
                });
                self.engine.execute(Command::SelectEntity { id }.into())?
            }
            Command::SetEditorMeshView { view } => {
                if self.mesh_view != *view {
                    self.mesh_view = *view;
                    self.session_revision += 1;
                }
                json!({"mesh_view":self.mesh_view})
            }
            Command::SelectEditorMeshRegion {
                instance,
                reference,
                component,
                min,
                max,
                width,
                height,
                additive,
            } => {
                if reference != &self.engine.mesh_reference() {
                    return Err(invalid(
                        "Mesh selection reference is stale; inspect the current revision",
                    ));
                }
                let camera = self
                    .editor
                    .current()
                    .camera
                    .as_ref()
                    .unwrap_or(&self.engine.scene.camera);
                let next = crate::mesh_screen::region(
                    &self.engine,
                    instance,
                    *component,
                    *min,
                    *max,
                    *width,
                    *height,
                    self.mesh_view.xray,
                    camera,
                )?;
                let asset = self
                    .engine
                    .scene
                    .meshes
                    .instances
                    .iter()
                    .find(|i| &i.id == instance)
                    .expect("region validated instance")
                    .asset
                    .clone();
                let previous = self
                    .engine
                    .selected_mesh_components()
                    .filter(|s| s.asset == asset)
                    .map(|s| s.components.clone());
                let components = crate::mesh_screen::combine(previous, next, *additive);
                let selection = components.map(|components| crate::mesh_topology::Selection {
                    asset,
                    reference: reference.clone(),
                    components,
                });
                let mut result = self
                    .engine
                    .execute(Command::SelectMeshComponents { selection }.into())?;
                self.select_mesh(Some(instance.clone()))?;
                result["view_revision"] = json!(self.engine.view_revision);
                result
            }
            Command::InspectEditor => {
                json!({"transform":self.transform,"mesh_view":self.mesh_view,"editor":self.editor,"scene_path":self.scene_path,"selected_mesh":self.selected_mesh,"character_editor":self.character_editor.inspect(&self.engine.scene),"native_window_required":false})
            }
            Command::SetViewportPerformance { settings } => {
                self.viewport_performance.configure(*settings);
                self.session_revision += 1;
                self.viewport_performance.inspect()
            }
            Command::InspectViewportPerformance => self.viewport_performance.inspect(),
            Command::CompileMeshLods {path} | Command::LoadMeshLods {path} => {
                if self.engine.playing {return Err(invalid("Pause before compiling/loading mesh LODs"));}
                if Path::new(path).extension().and_then(|x|x.to_str())!=Some("ignis-lod") {return Err(invalid("Mesh LOD sidecars require .ignis-lod extension"));}
                let compiling=matches!(&request.command,Command::CompileMeshLods{..});
                let started=std::time::Instant::now();
                let document=if compiling {
                    let d=crate::mesh_lods::compile(&self.engine.scene.meshes)?;
                    // Certify the exact serialized representation before either file or
                    // session publication. Failed candidates leave both untouched.
                    if !d.packs.is_empty() {
                        let bytes=d.encode();
                        let verified=crate::mesh_lods::Document::decode(&bytes,&self.engine.scene.meshes)?;
                        self.write(path,&bytes)?;verified
                    } else {d}
                } else {
                    let dest=self.resolve(path)?;let file=std::fs::File::open(dest).map_err(io_error)?;
                    let mut bytes=Vec::new();file.take(crate::mesh_lods::MAX_BYTES as u64+1).read_to_end(&mut bytes).map_err(io_error)?;
                    crate::mesh_lods::Document::decode(&bytes,&self.engine.scene.meshes)?
                };
                let output_written=compiling&&!document.packs.is_empty();
                let data=document.inspect();self.engine.mesh_lods=std::sync::Arc::new(document);
                self.engine.view_revision+=1;self.session_revision+=1;
                json!({"compiled":data,"output_written":output_written,"path":path,"preparation_ms":started.elapsed().as_secs_f64()*1000.,"session_only":true})
            }
            Command::EditEditor {
                action,
                expected_editor_revision,
            } => {
                if expected_editor_revision.is_some_and(|r| r != self.editor.revision) {
                    return Err(invalid("Editor revision conflict"));
                }
                self.editor.apply(action.clone())?;
                json!({"editor":self.editor})
            }
            Command::SaveEditor { path } => {
                let bytes = serde_json::to_vec_pretty(&self.editor).map_err(io_error)?;
                let dest = self.write(path, &bytes)?;
                json!({"path":dest,"editor_revision":self.editor.revision})
            }
            Command::LoadEditor { path } => {
                let mut editor: crate::editor::State = self.read(path)?;
                editor.validate()?;
                editor.version = 3;
                editor.revision = self
                    .editor
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| invalid("Editor revision exhausted"))?;
                self.editor = editor;
                json!({"editor":self.editor})
            }
            Command::CaptureEditor {
                path,
                width,
                height,
            } => {
                self.validate_capture(path, *width, *height)?;
                if *width < 640 || *height < 480 {
                    return Err(invalid("Editor capture requires at least640x480"));
                }
                let mut graphics = match self.editor_capture.take() {
                    Some(g) => g,
                    None => crate::editor_ui::Graphics::offscreen().map_err(io_error)?,
                };
                let captured = graphics.capture(self, *width, *height);
                self.editor_capture = Some(graphics);
                let bytes = captured.map_err(io_error)?;
                let dest = self.write(path, &bytes)?;
                return Ok(Reply {
                    data: json!({"path":dest,"width":width,"height":height,"mime_type":"image/png","editor_revision":self.editor.revision,"scene_revision":self.engine.revision,"native_window":false}),
                    png: Some(bytes),
                });
            }
            Command::CompareImages {
                reference_path,
                candidate_path,
                mask_path,
                regions,
                difference_path,
            } => {
                let sources: Vec<_> = [
                    Some(reference_path),
                    Some(candidate_path),
                    mask_path.as_ref(),
                ]
                .into_iter()
                .flatten()
                .map(|p| self.resolve(p))
                .collect::<Result<_>>()?;
                if let Some(path) = difference_path {
                    let output = destination_key(&self.resolve(path)?)?;
                    for source in &sources {
                        if output == destination_key(source)?
                            || output == source.canonicalize().map_err(io_error)?
                        {
                            return Err(invalid(
                                "Comparison difference output must not replace an input or mask",
                            ));
                        }
                    }
                }
                let read_png = |path: &Path| -> Result<crate::image_similarity::Image> {
                    let file = std::fs::File::open(path).map_err(io_error)?;
                    let mut bytes = Vec::new();
                    file.take((crate::image_similarity::MAX_BYTES + 1) as u64)
                        .read_to_end(&mut bytes)
                        .map_err(io_error)?;
                    crate::image_similarity::Image::decode(&bytes)
                };
                let reference = read_png(&sources[0])?;
                let candidate = read_png(&sources[1])?;
                let mask = sources
                    .get(2)
                    .map(|p| read_png(p)?.mask(reference.width, reference.height))
                    .transpose()?;
                let comparison = crate::image_similarity::compare(
                    &reference,
                    &candidate,
                    mask.as_deref(),
                    regions,
                    difference_path.is_some(),
                )?;
                let difference = difference_path
                    .as_ref()
                    .zip(comparison.difference_png.as_ref())
                    .map(|(path, bytes)| self.write(path, bytes))
                    .transpose()?;
                json!({"report":comparison.report,"reference_path":reference_path,"candidate_path":candidate_path,"mask_path":mask_path,
                    "difference_path":difference,"build_id":env!("IGNIS_BUILD_ID")})
            }
            Command::SaveScenario { path, scenario } => {
                scenario.validate(&self.engine.scene)?;
                let bytes = document_bytes(scenario)?;
                let dest = self.write(path, &bytes)?;
                json!({"path":dest,"saved":"verification_scenario","version":scenario.version})
            }
            Command::InspectScenario { path } => {
                let scenario: crate::scenario::Scenario = self.read(path)?;
                scenario.validate(&self.engine.scene)?;
                json!({"scenario":scenario})
            }
            Command::RunScenario {
                path,
                report_path,
                replay_path,
            } => {
                let source = self.resolve(path)?;
                let mut destinations = vec![
                    destination_key(&source)?,
                    source.canonicalize().map_err(io_error)?,
                ];
                for output in [report_path, replay_path].into_iter().flatten() {
                    let key = destination_key(&self.resolve(output)?)?;
                    if destinations.contains(&key) {
                        return Err(invalid(
                            "Scenario input, report and replay must use different destinations",
                        ));
                    }
                    destinations.push(key);
                }
                let scenario: crate::scenario::Scenario = self.read(path)?;
                let outcome = scenario.run(&self.engine.scene)?;
                let report = serde_json::to_value(&outcome.report).map_err(io_error)?;
                let report_dest = report_path
                    .as_ref()
                    .map(|output| self.write(output, &document_bytes(&report)?))
                    .transpose()?;
                let replay_dest = replay_path
                    .as_ref()
                    .map(|output| {
                        self.write(output, &document_bytes(&outcome.engine.recorded_replay()?)?)
                    })
                    .transpose()?;
                json!({"report":report,"report_path":report_dest,"replay_path":replay_dest})
            }
            Command::SavePortableSave { path } => {
                if self.network_client.is_some() {
                    return Err(invalid(
                        "Remote client has no authoritative portable save; disconnect first",
                    ));
                }
                if !path.ends_with(".ignis-portable") {
                    return Err(invalid(
                        "Portable save path requires .ignis-portable suffix",
                    ));
                }
                self.resolve(path)?;
                let saved = crate::portable_save::PortableSave::capture(&self.engine)?;
                let bytes = saved.encode()?;
                let destination = self.write(path, &bytes)?;
                json!({"path":destination,"tick":saved.tick(),"bytes":bytes.len(),"format":crate::portable_save::VERSION,"history_required":false,"physical_primaries":"ignis"})
            }
            Command::LoadPortableSave { path, mode } => {
                if self.network_host.is_some() || self.network_client.is_some() {
                    return Err(invalid(
                        "Stop multiplayer host/client explicitly before loading portable save",
                    ));
                }
                let bytes = crate::portable_save::PortableSave::read(&self.resolve(path)?)?;
                self.engine.load_portable_save(&bytes, *mode)?;
                self.scene_path = None;
                json!({"tick":self.engine.simulation.tick,"baseline_tick":self.engine.baseline_tick(),"revision":self.engine.revision,"mode":self.engine.execution_mode(),"external_controls":"renewed_and_neutral","contact_resume":"reconstructed"})
            }
            Command::InspectPortableSave { path } => {
                if let Some(path) = path {
                    let bytes = crate::portable_save::PortableSave::read(&self.resolve(path)?)?;
                    let candidate = Engine::from_portable_save(&bytes, None)?;
                    json!({"format":crate::portable_save::VERSION,"scene_version":crate::model::SCENE_VERSION,"spatial_version":crate::world_space::VERSION,"origin":crate::portable_save::origin(),"tick":candidate.simulation.tick,"mode":candidate.execution_mode(),"bytes":bytes.len(),"validated":true,"physical_primaries":"ignis"})
                } else {
                    json!({"format":crate::portable_save::VERSION,"max_bytes":crate::portable_save::MAX_BYTES,"spatial_version":crate::world_space::VERSION,"origin":crate::portable_save::origin(),"tick":self.engine.simulation.tick,"adapter":"rapier_primaries_v1","contact_resume":"reconstructed","history_required":false,"cross_backend_bitexact":false})
                }
            }
            Command::MigratePortableSave { source_path, path } => {
                if !path.ends_with(".ignis-portable") {
                    return Err(invalid(
                        "Portable migration destination requires .ignis-portable suffix",
                    ));
                }
                self.resolve(path)?;
                let source = crate::portable_save::PortableSave::read(&self.resolve(source_path)?)?;
                let bytes = crate::portable_save::PortableSave::migrate(&source)?;
                let destination = self.write(path, &bytes)?;
                json!({"path":destination,"bytes":bytes.len(),"format":crate::portable_save::VERSION,"world_changed":false,"validated":true})
            }
            Command::LoadPlanetRuntime{path}=>{
                let config:crate::planet::runtime::Config=self.read(path)?;
                self.engine.execute(Command::SetPlanetRuntime{config:Some(Box::new(config))}.into())?
            },
            Command::SaveCheckpoint { path } => {
                if self.network_client.is_some() {
                    return Err(invalid(
                        "Remote client has no authoritative local checkpoint; disconnect first",
                    ));
                }
                if !path.ends_with(".ignis-save") {
                    return Err(invalid("Checkpoint path requires .ignis-save suffix"));
                }
                self.resolve(path)?;
                let checkpoint = crate::checkpoint::Checkpoint::capture(&self.engine)?;
                let bytes = checkpoint.encode()?;
                let dest = self.write(path, &bytes)?;
                json!({"path":dest,"tick":checkpoint.tick(),"bytes":bytes.len(),"format":crate::checkpoint::VERSION,"history_required":false})
            }
            Command::LoadCheckpoint { path, mode } => {
                if self.network_host.is_some() || self.network_client.is_some() {
                    return Err(invalid(
                        "Stop multiplayer host/client explicitly before loading checkpoint",
                    ));
                }
                let bytes = crate::checkpoint::Checkpoint::read(&self.resolve(path)?)?;
                self.engine.load_checkpoint(&bytes, *mode)?;
                self.scene_path = None;
                json!({"tick":self.engine.simulation.tick,"baseline_tick":self.engine.baseline_tick(),"revision":self.engine.revision,"mode":self.engine.execution_mode(),"external_controls":"renewed_and_neutral"})
            }
            Command::SaveScene { path } => {
                if self
                    .project
                    .as_ref()
                    .is_some_and(|m| !m.scenes.contains(path))
                {
                    return Err(invalid(
                        "Crie a cena no projeto antes de salvar neste caminho",
                    ));
                }
                let dest = self.write_scene(path)?;
                self.scene_path = Some(path.clone());
                self.saved_scene_revision = self.engine.revision;
                json!({"path":dest,"revision":self.engine.revision,"saved":"authoring_scene"})
            }
            Command::LoadScene { path } => {
                if self
                    .project
                    .as_ref()
                    .is_some_and(|m| !m.scenes.contains(path))
                {
                    return Err(invalid("Scene is not registered in this project"));
                }
                let scene = self.read_scene(path)?;
                self.engine.replace(scene)?;
                self.scene_path = Some(path.clone());
                self.saved_scene_revision = self.engine.revision;
                json!({"revision":self.engine.revision,"tick":0})
            }
            Command::CaptureNpcAudio {
                path,
                frames,
                listener,
                radius,
            } => {
                if self.network_client.is_some() {
                    return Err(invalid("Remote presentation does not carry PCM audio"));
                }
                self.resolve(path)?;
                let pcm = crate::npc_presentation::pcm(
                    &self.engine.scene.avatar_audio,
                    &self.engine.simulation.presentation.voices,
                    self.engine.simulation.tick,
                    *frames,
                    *listener,
                    *radius,
                )?;
                let bytes = crate::npc_presentation::wav(&pcm);
                let dest = self.write(path, &bytes)?;
                json!({"path":dest,"frames":frames,"rate":crate::npc_presentation::RATE,"bytes":bytes.len(),"peak":pcm.iter().map(|x|x.unsigned_abs()).max().unwrap_or(0)})
            }
            Command::SaveReplay { path } => {
                let bytes = document_bytes(&self.engine.recorded_replay()?)?;
                let dest = self.write(path, &bytes)?;
                json!({"path":dest,"ticks":self.engine.simulation.tick})
            }
            Command::RunReplay { path } => Engine::verify_replay(&self.read::<Replay>(path)?)?,
            Command::InspectReplay { path, tick } => {
                let replay: Replay = self.read(path)?;
                let mut engine = Engine::replay_at(&replay, *tick)?;
                engine.inherit_view(&self.engine);
                json!({"snapshot":engine.snapshot(),
                    "noises":engine.execute(Command::InspectNoises.into())?,
                    "replay":replay_metadata(&replay, &engine, path)})
            }
            Command::CaptureReplay {
                replay_path,
                path,
                tick,
                width,
                height,
            } => {
                self.validate_capture(path, *width, *height)?;
                let source = self.resolve(replay_path)?;
                let output = destination_key(&self.resolve(path)?)?;
                if output == destination_key(&source)?
                    || output == source.canonicalize().map_err(io_error)?
                {
                    return Err(invalid(
                        "Replay and capture must use different destinations",
                    ));
                }
                let replay: Replay = self.read(replay_path)?;
                let mut engine = Engine::replay_at(&replay, *tick)?;
                engine.inherit_view(&self.engine);
                let mut reply =
                    self.capture_frame(&engine.render_frame(), path, *width, *height)?;
                reply.data["tick"] = json!(engine.simulation.tick);
                reply.data["revision"] = json!(engine.revision);
                reply.data["debug_view"] = json!(engine.debug_view);
                reply.data["selection"] = json!(engine.selected_id());
                reply.data["replay"] = replay_metadata(&replay, &engine, replay_path);
                return Ok(reply);
            }
            Command::Capture {
                path,
                width,
                height,
                clip,
            } => {
                self.validate_capture(path, *width, *height)?;
                if let Some(c)=clip {c.validate()?;if self.engine.planet_active(){return Err(invalid("Explicit capture clipping is for local scenes; preserve the planetary projection"));}}
                self.engine.prepare_planet_viewport(*width,*height)?;
                let mut frame=self.game_frame();
                if let Some(c)=clip {frame.clip=Some(*c);}
                let mut reply = self.capture_frame(&frame, path, *width, *height)?;
                if clip.is_some(){reply.data["clip"]=json!(clip);}
                reply.data["tick"] = json!(self.engine.simulation.tick);
                reply.data["revision"] = json!(self.engine.revision);
                reply.data["debug_view"] = json!(self.engine.debug_view);
                reply.data["selection"] = json!(self.engine.selected_id());
                return Ok(reply);
            }
            _ => self.engine.execute(request)?,
        };
        if clear_botany {
            self.botany.preview = None;
            self.botany.physics = None;
            self.botany.overlay = None;
            self.botany.basis = None;
            self.botany.revision = None;
        }
        self.refresh_botany()?;
        let invalidate = clear_mesh
            || self.selected_mesh.as_ref().is_some_and(|id| {
                !self
                    .engine
                    .scene
                    .meshes
                    .instances
                    .iter()
                    .any(|i| &i.id == id)
            });
        if invalidate && self.selected_mesh.take().is_some() {
            self.session_revision = self.session_revision.wrapping_add(1);
        }
        if session_change {
            self.session_revision = self.session_revision.wrapping_add(1);
        }
        self.character_editor.choose(None, &self.engine.scene);
        self.character_editor
            .refresh(&self.engine.scene, self.engine.revision);
        if self.audio_output.is_some() {
            self.sync_audio_scope();
            if pause_audio {
                self.audio_output.as_ref().unwrap().invalidate();
            }
            if self.remote_client() {
                self.audio_output = None;
            }
        }
        Ok(Reply { data, png: None })
    }
    fn validate_capture(&self, path: &str, width: u32, height: u32) -> Result<()> {
        self.resolve(path)?;
        if !(64..=2048).contains(&width) || !(64..=2048).contains(&height) {
            return Err(invalid("Capture width/height must be in [64,2048]"));
        }
        Ok(())
    }
    fn capture_frame(
        &mut self,
        frame: &RenderFrame,
        path: &str,
        width: u32,
        height: u32,
    ) -> Result<Reply> {
        if self.capture_renderer.is_none() {
            let instance = wgpu::Instance::new(&Default::default());
            self.capture_renderer = Some(
                pollster::block_on(Renderer::new(&instance, None)).map_err(|e| Error {
                    code: "graphics_unavailable",
                    message: e.to_string(),
                })?,
            );
        }
        let renderer = self.capture_renderer.as_mut().unwrap();
        let bytes = renderer.capture(frame, width, height).map_err(|e| Error {
            code: "capture_failed",
            message: e.to_string(),
        })?;
        let adapter = renderer.info();
        let dest = self.write(path, &bytes)?;
        let data = json!({"path":dest,"mime_type":"image/png","width":width,"height":height,
            "adapter":adapter,"camera":frame.camera,"entity_count":frame.boxes.len(),
            "mesh_vertex_count":frame.meshes.triangle_count()*3,"mesh_triangle_count":frame.meshes.triangle_count(),"mesh_resident_vertex_count":frame.meshes.resident_vertex_count(),
            "vision_guide_count":frame.vision_guides.len(),"hearing_guide_count":frame.hearing_guides.len(),
            "noise_guide_count":frame.noise_guides.len(),"route_guide_count":frame.route_guides.len(),
            "selection_guide_count":usize::from(frame.selection.is_some() || frame.selection_mesh.is_some()),"truncated_navigation_paths":frame.truncated_navigation_paths});
        Ok(Reply {
            data,
            png: Some(bytes),
        })
    }
}

fn replay_metadata(replay: &Replay, engine: &Engine, path: &str) -> Value {
    let at_end = engine.simulation.tick - engine.baseline_tick() == replay.inputs.len() as u64;
    json!({"path":path,"selected_tick":engine.simulation.tick-engine.baseline_tick(),"absolute_tick":engine.simulation.tick,"baseline_tick":engine.baseline_tick(),"total_ticks":replay.inputs.len(),
        "at_end":at_end,"final_verification":at_end.then(|| engine.compare_replay(replay)),
        "input_semantics":"last_applied_or_default","manual_noises_at_selected_tick":"pending"})
}
