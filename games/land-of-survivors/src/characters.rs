//! Game-owned local identities. Network leases never replace these durable IDs.
use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum People {
    HighElf,
    Orc,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Body {
    Feminine,
    Masculine,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Mage,
    Warrior,
    Archer,
    Druid,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Faction {
    Elves,
    Orcs,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub face: u8,
    pub skin: u8,
    pub eyes: u8,
    pub ears: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub name: String,
    pub people: People,
    pub body: Body,
    pub class: Class,
    pub faction: Faction,
    pub appearance: Appearance,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            name: String::new(),
            people: People::HighElf,
            body: Body::Feminine,
            class: Class::Mage,
            faction: Faction::Elves,
            appearance: Appearance {
                face: 0,
                skin: 1,
                eyes: 0,
                ears: 1,
            },
        }
    }
}
impl Draft {
    pub fn validate(&self) -> Result<()> {
        let name = self.name.trim();
        ensure!(
            (2..=24).contains(&name.chars().count()),
            "Use a name with 2–24 characters."
        );
        ensure!(
            name.chars()
                .all(|c| c.is_alphabetic() || c == ' ' || c == '-' || c == '\''),
            "Names may contain letters, spaces, apostrophes and hyphens."
        );
        ensure!(
            name.chars().filter(|c| c.is_alphabetic()).count() >= 2,
            "Use at least two letters in the name."
        );
        ensure!(
            !name.contains("  "),
            "Remove repeated spaces from the name."
        );
        ensure!(
            self.appearance.face < 3
                && self.appearance.skin < 4
                && self.appearance.eyes < 4
                && self.appearance.ears < 3,
            "Unsupported appearance option."
        );
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub id: String,
    pub identity: Draft,
    pub source_sha256: String,
    pub level: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Roster {
    pub version: u16,
    pub portuguese: bool,
    pub selected: Option<String>,
    pub characters: Vec<Character>,
}
impl Default for Roster {
    fn default() -> Self {
        Self {
            version: 1,
            portuguese: false,
            selected: None,
            characters: vec![],
        }
    }
}
impl Roster {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1 && self.characters.len() <= 64,
            "Unsupported or oversized character roster."
        );
        let mut ids = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for c in &self.characters {
            c.identity.validate()?;
            ensure!(
                c.id.len() == 32
                    && c.id.bytes().all(|b| b.is_ascii_hexdigit())
                    && ids.insert(&c.id),
                "Invalid or duplicate character identity."
            );
            ensure!(
                c.level > 0
                    && c.source_sha256.len() == 64
                    && c.source_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
                "Invalid character metadata."
            );
            ensure!(
                names.insert(c.identity.name.trim().to_lowercase()),
                "Duplicate character name."
            );
        }
        ensure!(
            self.selected.as_ref().is_none_or(|s| ids.contains(s)),
            "Selected character does not exist."
        );
        Ok(())
    }
}
pub struct Store {
    pub roster: Roster,
    path: PathBuf,
    stamp: Option<Vec<u8>>,
}
/// One offline world writer per durable character, including across processes.
pub struct SaveLease {
    path: PathBuf,
    _file: fs::File,
}
impl SaveLease {
    pub fn acquire(directory: &Path, id: &str) -> Result<Self> {
        ensure!(
            id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()),
            "Invalid character identity"
        );
        let directory = directory.join("saves");
        fs::create_dir_all(&directory)?;
        let path = directory.join(format!("{id}.lease"));
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| {
                anyhow::anyhow!(
                    "This character's offline save is already in use ({}): {e}",
                    path.display()
                )
            })?;
        Ok(Self { path, _file: file })
    }
}
impl Drop for SaveLease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
impl Store {
    pub fn default_path() -> PathBuf {
        if let Some(path) = std::env::var_os("LOS_PROFILE_PATH") {
            return path.into();
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        #[cfg(target_os = "macos")]
        let base = home.join("Library/Application Support");
        #[cfg(not(target_os = "macos"))]
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"));
        base.join("Ignis/LandOfSurvivors/characters.json")
    }
    pub fn load(path: PathBuf) -> Result<Self> {
        let bytes = read(&path)?;
        let roster: Roster = match &bytes {
            Some(b) => serde_json::from_slice(b)?,
            None => Roster::default(),
        };
        roster.validate()?;
        Ok(Self {
            roster,
            path,
            stamp: bytes,
        })
    }
    fn commit(&mut self, candidate: Roster) -> Result<()> {
        candidate.validate()?;
        let parent = self
            .path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Missing profile directory"))?;
        fs::create_dir_all(parent)?;
        let lock_path = self.path.with_extension("json.lock");
        let lock = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|e| {
                anyhow::anyhow!("Character storage is locked ({}): {e}", lock_path.display())
            })?;
        struct Lock(PathBuf, fs::File);
        impl Drop for Lock {
            fn drop(&mut self) {
                let _ = self.1.sync_all();
                let _ = fs::remove_file(&self.0);
            }
        }
        let _lock = Lock(lock_path, lock);
        ensure!(
            read(&self.path)? == self.stamp,
            "Characters changed in another window. Reopen this window before saving."
        );
        let bytes = serde_json::to_vec_pretty(&candidate)?;
        ensure!(
            bytes.len() <= 1024 * 1024,
            "Roster exceeds its storage budget."
        );
        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        tmp.write_all(&bytes)?;
        tmp.as_file().sync_all()?;
        tmp.persist(&self.path)?;
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        self.stamp = Some(bytes);
        self.roster = candidate;
        Ok(())
    }
    pub fn create(&mut self, mut draft: Draft, source_sha256: String) -> Result<String> {
        draft.name = draft.name.trim().into();
        draft.validate()?;
        ensure!(
            !self
                .roster
                .characters
                .iter()
                .any(|c| c.identity.name.to_lowercase() == draft.name.to_lowercase()),
            "A character with that name already exists."
        );
        let mut entropy = [0u8; 16];
        getrandom::fill(&mut entropy).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let id = entropy
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let mut next = self.roster.clone();
        next.characters.push(Character {
            id: id.clone(),
            identity: draft,
            source_sha256,
            level: 1,
        });
        next.selected = Some(id.clone());
        self.commit(next)?;
        Ok(id)
    }
    pub fn select(&mut self, id: &str) -> Result<()> {
        let mut next = self.roster.clone();
        next.selected = Some(id.into());
        self.commit(next)
    }
    pub fn language(&mut self, portuguese: bool) -> Result<()> {
        let mut next = self.roster.clone();
        next.portuguese = portuguese;
        self.commit(next)
    }
    pub fn cosmetics(&mut self, id: &str, appearance: Appearance) -> Result<()> {
        let mut next = self.roster.clone();
        let c = next
            .characters
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or_else(|| anyhow::anyhow!("Unknown character"))?;
        c.identity.appearance = appearance;
        self.commit(next)
    }
    pub fn directory(&self) -> &Path {
        self.path.parent().unwrap()
    }
    pub fn selected(&self) -> Option<&Character> {
        self.roster
            .characters
            .iter()
            .find(|c| Some(&c.id) == self.roster.selected.as_ref())
    }
}
fn read(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::metadata(path) {
        Ok(meta) => {
            ensure!(
                meta.is_file() && meta.len() <= 1024 * 1024,
                "Invalid or oversized character file."
            );
            Ok(Some(fs::read(path)?))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => bail!(e),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_cancel_cosmetics_and_corruption_are_safe() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("characters.json");
        let mut s = Store::load(path.clone())?;
        let d = Draft {
            name: "Aelara".into(),
            ..Default::default()
        };
        let mut cancelled = d.clone();
        cancelled.name = "Discarded".into();
        drop(cancelled);
        assert!(!path.exists());
        let id = s.create(d.clone(), "a".repeat(64))?;
        assert!(s.create(d, "a".repeat(64)).is_err());
        let before = s.selected().unwrap().clone();
        let mut appearance = before.identity.appearance.clone();
        appearance.face = 2;
        s.cosmetics(&id, appearance)?;
        let c = s.selected().unwrap();
        let mut restored_identity = c.identity.clone();
        restored_identity.appearance = before.identity.appearance.clone();
        assert_eq!(restored_identity, before.identity);
        assert_eq!(c.id, before.id);
        assert_eq!(c.identity.class, before.identity.class);
        assert_eq!(c.identity.people, before.identity.people);
        let reopened = Store::load(path.clone())?;
        assert_eq!(reopened.roster, s.roster);
        fs::write(&path, b"{broken")?;
        assert!(s.select(&id).is_err());
        assert_eq!(fs::read(&path)?, b"{broken");
        assert!(Store::load(path).is_err());
        Ok(())
    }
}
