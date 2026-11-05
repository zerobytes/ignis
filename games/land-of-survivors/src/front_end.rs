use crate::{characters::*, presentation::Presentation};
use anyhow::{Result, ensure};
use ignis::{
    engine::Command,
    meshes::character::{Configuration, Placement, Source},
    model::Scene,
    service::Service,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::PathBuf};

pub fn command(s: &mut Service, value: Value) -> Result<Value> {
    Ok(s.execute(serde_json::from_value::<Command>(value)?.into())?
        .data)
}
pub fn digest(bytes: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Title,
    Roster,
    Create,
    Cosmetics,
    Session,
}
#[derive(Clone, Debug)]
pub enum Action {
    Roster,
    New,
    Cancel,
    Create,
    Select(String),
    Preview,
    Cosmetics,
    SaveCosmetics,
    Language(bool),
    Offline,
    Host,
    Join,
    Leave,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invite {
    pub version: u16,
    pub address: String,
    pub certificate_der: String,
}
pub struct Session {
    pub host: Option<Service>,
    pub client: Service,
    pub offline: bool,
    pub character: String,
    pub invitation: String,
    pub status: Value,
    pub visuals: Presentation,
    elapsed: f32,
    inspect_elapsed: f32,
    _save_lease: Option<SaveLease>,
}
pub struct Front {
    pub stage: Scene,
    pub preview: Service,
    pub store: Option<Store>,
    pub error: String,
    pub page: Page,
    pub draft: Draft,
    pub portuguese: bool,
    pub bind: String,
    pub invitation: String,
    pub session: Option<Session>,
    source: Option<Source>,
    pub source_hash: String,
    pub preview_error: String,
    pub yaw: f32,
}
impl Front {
    pub fn new(engine_root: PathBuf, path: PathBuf) -> Result<Self> {
        let stage: Scene = serde_json::from_slice(&fs::read(
            engine_root.join("games/land-of-survivors/scenes/Inicio.scene.json"),
        )?)?;
        let preview = Service::new(&engine_root, stage.clone())?;
        let (store, error) = match Store::load(path) {
            Ok(s) => (Some(s), String::new()),
            Err(e) => (
                None,
                format!("Could not open characters; the existing file was preserved. {e}"),
            ),
        };
        let portuguese = store.as_ref().is_some_and(|s| s.roster.portuguese);
        let source_path = engine_root.join("artifacts/character-base/human-nose.ignis-character");
        let mut preview_error = String::new();
        let mut source_hash = String::new();
        let source = (|| -> Result<Source> {
            ensure!(
                fs::metadata(&source_path)?.len() <= ignis::meshes::character::MAX_BYTES as u64,
                "Character source too large"
            );
            let bytes = fs::read(source_path)?;
            let s: Source = serde_json::from_slice(&bytes)?;
            s.validate()?;
            source_hash = digest(&bytes);
            Ok(s)
        })();
        let source = match source {
            Ok(s) => Some(s),
            Err(e) => {
                preview_error = format!("Character preview unavailable: {e}");
                None
            }
        };
        Ok(Self {
            stage,
            preview,
            store,
            error,
            page: Page::Title,
            draft: Draft::default(),
            portuguese,
            bind: "127.0.0.1:43000".into(),
            invitation: String::new(),
            session: None,
            source,
            source_hash,
            preview_error,
            yaw: 0.,
        })
    }
    pub fn store_mut(&mut self) -> Result<&mut Store> {
        self.store.as_mut().ok_or_else(||anyhow::anyhow!("Character storage unavailable. Close and repair/reopen the character file before saving."))
    }
    pub fn selected(&self) -> Option<&Character> {
        self.store.as_ref().and_then(|s| s.selected())
    }
    pub fn execute(&mut self, action: Action) -> Result<()> {
        self.error.clear();
        match action {
            Action::Roster => {
                self.page = Page::Roster;
                if let Some(c) = self.selected() {
                    self.draft = c.identity.clone();
                    self.preview()?;
                }
            }
            Action::New => {
                self.draft = Draft::default();
                self.yaw = 0.;
                self.page = Page::Create;
                self.preview()?;
            }
            Action::Cancel => {
                self.page = Page::Roster;
                if let Some(c) = self.selected() {
                    self.draft = c.identity.clone();
                    self.preview()?;
                } else {
                    self.draft = Draft::default();
                    self.preview.engine.replace(self.stage.clone())?;
                }
            }
            Action::Preview => self.preview()?,
            Action::Create => {
                ensure!(
                    self.source.is_some(),
                    "Character source is required before creating a character."
                );
                let draft = self.draft.clone();
                let hash = self.source_hash.clone();
                self.store_mut()?.create(draft, hash)?;
                self.page = Page::Roster;
            }
            Action::Select(id) => {
                self.store_mut()?.select(&id)?;
                self.draft = self.selected().unwrap().identity.clone();
                self.preview()?;
            }
            Action::Cosmetics => {
                self.draft = self
                    .selected()
                    .ok_or_else(|| anyhow::anyhow!("Select a character"))?
                    .identity
                    .clone();
                self.page = Page::Cosmetics;
            }
            Action::SaveCosmetics => {
                let id = self.selected().unwrap().id.clone();
                let appearance = self.draft.appearance.clone();
                self.store_mut()?.cosmetics(&id, appearance)?;
                self.page = Page::Roster;
            }
            Action::Language(pt) => {
                self.store_mut()?.language(pt)?;
                self.portuguese = pt;
            }
            Action::Offline => self.offline()?,
            Action::Host => self.host()?,
            Action::Join => self.join()?,
            Action::Leave => self.leave()?,
        }
        Ok(())
    }
    pub fn character_meshes(&self, d: &Draft) -> Result<ignis::meshes::Document> {
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Character source is unavailable"))?;
        let mut p = BTreeMap::new();
        p.insert(
            "body_masculinity".into(),
            if d.body == Body::Masculine { 1. } else { 0. },
        );
        p.insert(
            "jaw_width".into(),
            if d.people == People::Orc {
                0.65
            } else {
                [-0.18, 0., 0.15][d.appearance.face as usize]
            },
        );
        p.insert(
            "nose_width".into(),
            if d.people == People::Orc {
                0.40
            } else {
                [-0.12, 0.08, 0.20][d.appearance.face as usize]
            },
        );
        for side in ["L", "R"] {
            p.insert(
                format!("ear_pointed_{side}"),
                if d.people == People::HighElf {
                    [0.6, 1.2, 1.8][d.appearance.ears as usize]
                } else {
                    0.25
                },
            );
        }
        let (mut doc, _) = source.compile(
            &Configuration { parameters: p },
            &Placement {
                prefix: "creation".into(),
                position: [0., 0., 0.],
                rotation: [0., 0., 0., 1.],
                scale: 1.,
            },
        )?;
        let skins = if d.people == People::Orc {
            [
                [0.13, 0.20, 0.08],
                [0.18, 0.27, 0.11],
                [0.24, 0.29, 0.12],
                [0.09, 0.14, 0.06],
            ]
        } else {
            [
                [0.65, 0.46, 0.33],
                [0.48, 0.30, 0.20],
                [0.27, 0.14, 0.08],
                [0.13, 0.06, 0.03],
            ]
        };
        let eyes = [
            [0.08, 0.28, 0.15],
            [0.10, 0.22, 0.42],
            [0.29, 0.13, 0.04],
            [0.40, 0.28, 0.07],
        ];
        for i in &mut doc.instances {
            if i.id.ends_with("_body") {
                i.color = skins[d.appearance.skin as usize];
            } else if i.id.ends_with("_iris") {
                i.color = eyes[d.appearance.eyes as usize];
            }
        }
        Ok(doc)
    }
    pub fn preview(&mut self) -> Result<()> {
        if self.source.is_none() {
            return Ok(());
        }
        let doc = self.character_meshes(&self.draft)?;
        let height = doc
            .assets
            .iter()
            .flat_map(|a| a.geometry.positions.iter())
            .map(|p| p[1])
            .fold(0f32, f32::max);
        let mut scene = self.stage.clone();
        scene.meshes = doc;
        scene.characters.clear();
        scene.camera = ignis::model::Camera {
            position: [self.yaw.sin() * 0.95, height - 0.13, -self.yaw.cos() * 0.95],
            target: [0., height - 0.15, 0.],
            fov_degrees: 33.,
        };
        scene.light = serde_json::from_value(
            json!({"direction":[0.5,-0.7,0.7],"ambient":0.34,"direct_intensity":1.1,"environment":{"sky":[0.4,0.48,0.5],"horizon":[0.4,0.4,0.4],"ground":[0.15,0.14,0.12],"axis":[0,1,0],"intensity":0.4}}),
        )?;
        self.preview.engine.replace(scene)?;
        Ok(())
    }
    fn identity(&self) -> Result<Character> {
        self.selected()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Select or create a character first."))
    }
    fn world(&self, character: &Character, count: usize) -> Result<Service> {
        let mut scene = self.stage.clone();
        scene.name = "Land of Survivors · session".into();
        for i in 0..count {
            let value = json!({"id":format!("player{i}"),"name":if i==0{character.identity.name.clone()}else{format!("Traveler {}",i+1)},"transform":{"position":[i as f32*2.,1.1,0.],"size":[0.6,1.8,0.6]},"body":"player","color":if i==0{[0.2,0.5,0.36]}else{[0.4,0.3,0.2]},"faction":if character.identity.faction==Faction::Elves{"elves"}else{"orcs"}});
            scene.entities.push(serde_json::from_value(value)?);
        }
        scene.camera =
            serde_json::from_value(json!({"position":[8,7,13],"target":[3,1,0],"fov_degrees":48}))?;
        let root = self.store.as_ref().unwrap().directory();
        fs::create_dir_all(root)?;
        Ok(Service::new(root, scene)?)
    }
    fn offline(&mut self) -> Result<()> {
        let c = self.identity()?;
        let lease = SaveLease::acquire(self.store.as_ref().unwrap().directory(), &c.id)?;
        let mut client = self.world(&c, 1)?;
        let path = format!("saves/{}.ignis-portable", c.id);
        if self
            .store
            .as_ref()
            .unwrap()
            .directory()
            .join(&path)
            .is_file()
        {
            command(&mut client, json!({"op":"load_portable_save","path":path}))?;
        }
        command(
            &mut client,
            json!({"op":"set_execution_mode","mode":"continuous"}),
        )?;
        command(&mut client, json!({"op":"play"}))?;
        ensure!(
            client.engine.scene.meshes.is_empty(),
            "This character stage requires an empty authored mesh layer"
        );
        let visuals = Presentation::new(self.character_meshes(&c.identity)?, None)?;
        self.session = Some(Session {
            host: None,
            client,
            visuals,
            offline: true,
            _save_lease: Some(lease),
            character: c.id,
            invitation: String::new(),
            status: json!({"state":"offline"}),
            elapsed: 0.,
            inspect_elapsed: 0.,
        });
        self.page = Page::Session;
        Ok(())
    }
    fn host(&mut self) -> Result<()> {
        let c = self.identity()?;
        let address: std::net::SocketAddr = self.bind.parse()?;
        ensure!(
            !address.ip().is_unspecified(),
            "Use a specific local/LAN IP address, not 0.0.0.0."
        );
        let mut host = self.world(&c, 8)?;
        command(
            &mut host,
            json!({"op":"start_multiplayer_host","address":self.bind,"capacity":8}),
        )?;
        let client = self.world(&c, 1)?;
        let visuals = Presentation::new(
            self.character_meshes(&c.identity)?,
            Some(self.character_meshes(&Draft::default())?),
        )?;
        self.session = Some(Session {
            visuals,
            host: Some(host),
            client,
            offline: false,
            _save_lease: None,
            character: c.id,
            invitation: String::new(),
            status: json!({"state":"starting"}),
            elapsed: 0.,
            inspect_elapsed: 0.,
        });
        self.page = Page::Session;
        Ok(())
    }
    fn join(&mut self) -> Result<()> {
        let c = self.identity()?;
        ensure!(self.invitation.len() <= 32768, "Invitation too long");
        let invite: Invite = serde_json::from_str(self.invitation.trim())?;
        ensure!(invite.version == 1, "Unsupported invitation");
        let mut client = self.world(&c, 1)?;
        command(
            &mut client,
            json!({"op":"connect_multiplayer","address":invite.address,"certificate_der":invite.certificate_der}),
        )?;
        let visuals = Presentation::new(
            self.character_meshes(&c.identity)?,
            Some(self.character_meshes(&Draft::default())?),
        )?;
        self.session = Some(Session {
            host: None,
            client,
            visuals,
            offline: false,
            _save_lease: None,
            character: c.id,
            invitation: String::new(),
            status: json!({"state":"connecting"}),
            elapsed: 0.,
            inspect_elapsed: 0.,
        });
        self.page = Page::Session;
        Ok(())
    }
    pub fn tick(&mut self, dt: f32, input: ignis::physics::Input) -> Result<()> {
        let Some(s) = &mut self.session else {
            return Ok(());
        };
        s.elapsed = (s.elapsed + dt).min(0.25);
        s.inspect_elapsed += dt;
        if let Some(host) = &mut s.host {
            host.poll_network()?;
            if s.invitation.is_empty() {
                let info = command(host, json!({"op":"inspect_multiplayer_host"}))?;
                if info["transport"]["state"] == "listening" {
                    let invite = Invite {
                        version: 1,
                        address: info["transport"]["address"].as_str().unwrap().into(),
                        certificate_der: info["transport"]["certificate_der"]
                            .as_str()
                            .unwrap()
                            .into(),
                    };
                    s.invitation = serde_json::to_string(&invite)?;
                    command(
                        &mut s.client,
                        json!({"op":"connect_multiplayer","address":invite.address,"certificate_der":invite.certificate_der}),
                    )?;
                } else if info["transport"]["state"] == "failed" {
                    anyhow::bail!("{}", info["transport"]["error"]);
                }
            }
        }
        while s.elapsed >= 1. / 60. {
            if let Some(host) = &mut s.host {
                host.advance(1)?;
            } else if s.offline {
                s.client.advance(1)?;
            }
            s.elapsed -= 1. / 60.;
        }
        s.client.poll_network()?;
        if s.offline {
            command(&mut s.client, json!({"op":"set_input","input":input}))?;
        } else if s.client.remote_client() {
            let _ = command(
                &mut s.client,
                json!({"op":"set_multiplayer_input","input":input}),
            );
        }
        if s.inspect_elapsed >= 0.25 {
            s.inspect_elapsed = 0.;
            if !s.offline {
                s.status = command(&mut s.client, json!({"op":"inspect_multiplayer_client"}))?;
            }
        }
        Ok(())
    }
    pub fn leave(&mut self) -> Result<()> {
        if let Some(s) = &mut self.session {
            if s.offline {
                command(
                    &mut s.client,
                    json!({"op":"set_input","input":{"x":0,"z":0}}),
                )?;
                let was_playing = s.client.engine.playing;
                command(&mut s.client, json!({"op":"pause"}))?;
                let saved = command(
                    &mut s.client,
                    json!({"op":"save_portable_save","path":format!("saves/{}.ignis-portable",s.character)}),
                );
                if saved.is_err() && was_playing {
                    let _ = command(&mut s.client, json!({"op":"play"}));
                }
                saved?;
            } else {
                command(&mut s.client, json!({"op":"disconnect_multiplayer"}))?;
                if let Some(host) = &mut s.host {
                    command(host, json!({"op":"stop_multiplayer_host"}))?;
                }
            }
        }
        self.session = None;
        self.page = Page::Roster;
        Ok(())
    }
    pub fn present(&mut self, frame: &mut ignis::render::RenderFrame) -> Result<()> {
        if let Some(s) = &mut self.session {
            let local = if s.offline {
                frame.player_primitives.first().map(|p| p.actor)
            } else {
                s.status["actor"]["actor"].as_u64()
            };
            s.visuals.apply(frame, local)?;
        }
        Ok(())
    }
    pub fn service(&self) -> &Service {
        self.session.as_ref().map_or(&self.preview, |s| &s.client)
    }
    pub fn selected_save_path(&self) -> Option<PathBuf> {
        self.selected().map(|c| {
            self.store
                .as_ref()
                .unwrap()
                .directory()
                .join(format!("saves/{}.ignis-portable", c.id))
        })
    }
}
