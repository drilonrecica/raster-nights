// SPDX-License-Identifier: MPL-2.0

//! Versioned storage schemas and host-independent persistence behavior.
//!
//! Host crates provide platform storage. This crate owns codecs, recovery
//! policy, and an in-memory adapter used by tests and persistence fallbacks.

use std::collections::BTreeMap;

use raster_engine::{
    PersistenceError, ScoreRecord, ScoreRepository, Settings, SettingsRepository, SystemState,
    SystemStateRepository,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

pub const SETTINGS_FORMAT_VERSION: u16 = 1;
pub const SCORES_FORMAT_VERSION: u16 = 1;
pub const SYSTEM_STATE_FORMAT_VERSION: u16 = 1;

const SETTINGS_DOMAIN: &str = "settings";
const SCORES_DOMAIN: &str = "scores";
const SYSTEM_STATE_DOMAIN: &str = "system state";

/// Logical records kept separate so one corrupt domain cannot erase another.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StorageKey {
    Settings,
    Scores,
    SystemState,
}

impl StorageKey {
    #[must_use]
    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Settings => "settings.toml",
            Self::Scores => "scores.json",
            Self::SystemState => "system-state.json",
        }
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct StorageError {
    message: String,
}

impl StorageError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Minimal byte-oriented boundary implemented by native and browser hosts.
pub trait ByteStorage {
    fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, StorageError>;
    fn write(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError>;
    fn remove(&mut self, key: StorageKey) -> Result<(), StorageError>;

    /// Preserve bytes rejected by a codec before the repository returns defaults.
    fn preserve_corrupt(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError>;
}

/// Volatile storage used by tests and as an explicit session-only fallback.
#[derive(Clone, Debug, Default)]
pub struct MemoryByteStorage {
    values: BTreeMap<StorageKey, Vec<u8>>,
    corrupt: BTreeMap<StorageKey, Vec<Vec<u8>>>,
}

impl MemoryByteStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_raw(&mut self, key: StorageKey, data: impl Into<Vec<u8>>) {
        self.values.insert(key, data.into());
    }

    #[must_use]
    pub fn raw(&self, key: StorageKey) -> Option<&[u8]> {
        self.values.get(&key).map(Vec::as_slice)
    }

    #[must_use]
    pub fn preserved_corrupt(&self, key: StorageKey) -> &[Vec<u8>] {
        self.corrupt.get(&key).map_or(&[], Vec::as_slice)
    }
}

impl ByteStorage for MemoryByteStorage {
    fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, StorageError> {
        Ok(self.values.get(&key).cloned())
    }

    fn write(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError> {
        self.values.insert(key, data.to_vec());
        Ok(())
    }

    fn remove(&mut self, key: StorageKey) -> Result<(), StorageError> {
        self.values.remove(&key);
        Ok(())
    }

    fn preserve_corrupt(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError> {
        self.corrupt.entry(key).or_default().push(data.to_vec());
        Ok(())
    }
}

/// Typed repositories backed by one byte-storage implementation.
#[derive(Clone, Debug)]
pub struct Repository<B> {
    storage: B,
}

pub type InMemoryRepository = Repository<MemoryByteStorage>;

impl<B> Repository<B> {
    #[must_use]
    pub const fn new(storage: B) -> Self {
        Self { storage }
    }

    #[must_use]
    pub const fn storage(&self) -> &B {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut B {
        &mut self.storage
    }

    #[must_use]
    pub fn into_inner(self) -> B {
        self.storage
    }
}

impl Default for Repository<MemoryByteStorage> {
    fn default() -> Self {
        Self::new(MemoryByteStorage::default())
    }
}

impl<B: ByteStorage> Repository<B> {
    pub fn reset_settings(&mut self) -> Result<(), PersistenceError> {
        self.remove(StorageKey::Settings, SETTINGS_DOMAIN)
    }

    pub fn reset_scores(&mut self) -> Result<(), PersistenceError> {
        self.remove(StorageKey::Scores, SCORES_DOMAIN)
    }

    pub fn reset_system_state(&mut self) -> Result<(), PersistenceError> {
        self.remove(StorageKey::SystemState, SYSTEM_STATE_DOMAIN)
    }

    fn remove(&mut self, key: StorageKey, domain: &'static str) -> Result<(), PersistenceError> {
        self.storage
            .remove(key)
            .map_err(|error| PersistenceError::WriteFailed {
                domain,
                message: error.to_string(),
            })
    }

    fn read(
        &self,
        key: StorageKey,
        domain: &'static str,
    ) -> Result<Option<Vec<u8>>, PersistenceError> {
        self.storage
            .read(key)
            .map_err(|error| PersistenceError::Unavailable(format!("{domain}: {error}")))
    }

    fn write(
        &mut self,
        key: StorageKey,
        domain: &'static str,
        data: &[u8],
    ) -> Result<(), PersistenceError> {
        self.storage
            .write(key, data)
            .map_err(|error| PersistenceError::WriteFailed {
                domain,
                message: error.to_string(),
            })
    }

    fn corrupt(
        &mut self,
        key: StorageKey,
        domain: &'static str,
        data: &[u8],
        message: impl Into<String>,
    ) -> PersistenceError {
        let message = message.into();
        match self.storage.preserve_corrupt(key, data) {
            Ok(()) => PersistenceError::CorruptData { domain, message },
            Err(error) => PersistenceError::CorruptData {
                domain,
                message: format!("{message}; corrupt data could not be preserved: {error}"),
            },
        }
    }

    fn decode_json<T: DeserializeOwned>(
        &mut self,
        key: StorageKey,
        domain: &'static str,
        supported_version: u16,
        data: &[u8],
    ) -> Result<T, PersistenceError> {
        let version = match serde_json::from_slice::<VersionProbe>(data) {
            Ok(probe) => probe.format_version,
            Err(error) => return Err(self.corrupt(key, domain, data, error.to_string())),
        };
        ensure_version(domain, version, supported_version)?;
        serde_json::from_slice(data)
            .map_err(|error| self.corrupt(key, domain, data, error.to_string()))
    }

    fn decode_toml<T: DeserializeOwned>(
        &mut self,
        key: StorageKey,
        domain: &'static str,
        supported_version: u16,
        data: &[u8],
    ) -> Result<T, PersistenceError> {
        let text = match std::str::from_utf8(data) {
            Ok(text) => text,
            Err(error) => return Err(self.corrupt(key, domain, data, error.to_string())),
        };
        let version = match toml::from_str::<VersionProbe>(text) {
            Ok(probe) => probe.format_version,
            Err(error) => return Err(self.corrupt(key, domain, data, error.to_string())),
        };
        ensure_version(domain, version, supported_version)?;
        toml::from_str(text).map_err(|error| self.corrupt(key, domain, data, error.to_string()))
    }
}

impl<B: ByteStorage> SettingsRepository for Repository<B> {
    fn load_settings(&mut self) -> Result<Settings, PersistenceError> {
        let Some(data) = self.read(StorageKey::Settings, SETTINGS_DOMAIN)? else {
            return Ok(Settings::default());
        };
        let file: SettingsFileV1 = self.decode_toml(
            StorageKey::Settings,
            SETTINGS_DOMAIN,
            SETTINGS_FORMAT_VERSION,
            &data,
        )?;
        Ok(file.settings)
    }

    fn save_settings(&mut self, settings: &Settings) -> Result<(), PersistenceError> {
        let file = SettingsFileV1 {
            format_version: SETTINGS_FORMAT_VERSION,
            settings: settings.clone(),
        };
        let data =
            toml::to_string_pretty(&file).map_err(|error| PersistenceError::WriteFailed {
                domain: SETTINGS_DOMAIN,
                message: error.to_string(),
            })?;
        self.write(StorageKey::Settings, SETTINGS_DOMAIN, data.as_bytes())
    }
}

impl<B: ByteStorage> ScoreRepository for Repository<B> {
    fn load_scores(&mut self) -> Result<Vec<ScoreRecord>, PersistenceError> {
        let Some(data) = self.read(StorageKey::Scores, SCORES_DOMAIN)? else {
            return Ok(Vec::new());
        };
        let file: ScoresFileV1 = self.decode_json(
            StorageKey::Scores,
            SCORES_DOMAIN,
            SCORES_FORMAT_VERSION,
            &data,
        )?;
        Ok(file.records)
    }

    fn save_scores(&mut self, scores: &[ScoreRecord]) -> Result<(), PersistenceError> {
        let file = ScoresFileV1 {
            format_version: SCORES_FORMAT_VERSION,
            records: scores.to_vec(),
        };
        let mut data =
            serde_json::to_vec_pretty(&file).map_err(|error| PersistenceError::WriteFailed {
                domain: SCORES_DOMAIN,
                message: error.to_string(),
            })?;
        data.push(b'\n');
        self.write(StorageKey::Scores, SCORES_DOMAIN, &data)
    }
}

impl<B: ByteStorage> SystemStateRepository for Repository<B> {
    fn load_system_state(&mut self) -> Result<SystemState, PersistenceError> {
        let Some(data) = self.read(StorageKey::SystemState, SYSTEM_STATE_DOMAIN)? else {
            return Ok(SystemState::default());
        };
        let file: SystemStateFileV1 = self.decode_json(
            StorageKey::SystemState,
            SYSTEM_STATE_DOMAIN,
            SYSTEM_STATE_FORMAT_VERSION,
            &data,
        )?;
        Ok(file.state)
    }

    fn save_system_state(&mut self, state: &SystemState) -> Result<(), PersistenceError> {
        let file = SystemStateFileV1 {
            format_version: SYSTEM_STATE_FORMAT_VERSION,
            state: state.clone(),
        };
        let mut data =
            serde_json::to_vec_pretty(&file).map_err(|error| PersistenceError::WriteFailed {
                domain: SYSTEM_STATE_DOMAIN,
                message: error.to_string(),
            })?;
        data.push(b'\n');
        self.write(StorageKey::SystemState, SYSTEM_STATE_DOMAIN, &data)
    }
}

#[derive(Deserialize)]
struct VersionProbe {
    format_version: u16,
}

#[derive(Deserialize, Serialize)]
struct SettingsFileV1 {
    format_version: u16,
    #[serde(flatten)]
    settings: Settings,
}

#[derive(Deserialize, Serialize)]
struct ScoresFileV1 {
    format_version: u16,
    records: Vec<ScoreRecord>,
}

#[derive(Deserialize, Serialize)]
struct SystemStateFileV1 {
    format_version: u16,
    #[serde(flatten)]
    state: SystemState,
}

fn ensure_version(
    domain: &'static str,
    found: u16,
    supported: u16,
) -> Result<(), PersistenceError> {
    if found == supported {
        Ok(())
    } else {
        Err(PersistenceError::IncompatibleVersion {
            domain,
            found,
            supported,
        })
    }
}

#[cfg(test)]
mod tests {
    use raster_engine::{
        AssistanceProfileId, GameId, GameOutcome, ModeId, RulesRevision, RunSeed, SimulationTick,
        StateHash, ThreeCharacterTag,
    };

    use super::*;

    fn score(value: u64) -> ScoreRecord {
        ScoreRecord {
            game_id: GameId::parse("signal-stack").expect("valid fixture"),
            mode_id: ModeId::parse("standard-transmission").expect("valid fixture"),
            rules_revision: RulesRevision::new(1).expect("valid fixture"),
            assistance_profile: AssistanceProfileId::parse("canonical").expect("valid fixture"),
            tag: ThreeCharacterTag::parse("NUL").expect("valid fixture"),
            score: value,
            duration: SimulationTick(3_600),
            seed: RunSeed(42),
            outcome: GameOutcome::GameOver,
            final_state_hash: StateHash(0xDEAD_BEEF),
            recorded_at_unix_seconds: 1_700_000_000,
        }
    }

    #[test]
    fn all_domains_round_trip_in_their_documented_formats() {
        let mut repository = InMemoryRepository::default();
        let settings = Settings {
            reduced_motion: true,
            ..Settings::default()
        };
        let scores = vec![score(12_345)];
        let state = SystemState {
            privacy_acknowledged: true,
            last_selected_game: Some(GameId::parse("signal-stack").expect("valid fixture")),
            last_game_mode: Some(ModeId::parse("standard-transmission").expect("valid fixture")),
            last_score_tag: Some(ThreeCharacterTag::parse("NUL").expect("valid fixture")),
        };

        repository.save_settings(&settings).expect("settings save");
        repository.save_scores(&scores).expect("scores save");
        repository
            .save_system_state(&state)
            .expect("system-state save");

        assert_eq!(repository.load_settings().expect("settings load"), settings);
        assert_eq!(repository.load_scores().expect("scores load"), scores);
        assert_eq!(
            repository.load_system_state().expect("system-state load"),
            state
        );
        assert!(
            std::str::from_utf8(
                repository
                    .storage()
                    .raw(StorageKey::Settings)
                    .expect("settings bytes")
            )
            .expect("settings UTF-8")
            .contains("format_version = 1")
        );
        assert!(
            std::str::from_utf8(
                repository
                    .storage()
                    .raw(StorageKey::Scores)
                    .expect("score bytes")
            )
            .expect("scores UTF-8")
            .contains("\"format_version\": 1")
        );
    }

    #[test]
    fn missing_records_load_domain_defaults() {
        let mut repository = InMemoryRepository::default();
        assert_eq!(
            repository.load_settings().expect("default settings"),
            Settings::default()
        );
        assert!(repository.load_scores().expect("default scores").is_empty());
        assert_eq!(
            repository.load_system_state().expect("default state"),
            SystemState::default()
        );
    }

    #[test]
    fn corrupt_data_is_preserved_and_reported() {
        let mut storage = MemoryByteStorage::default();
        storage.insert_raw(StorageKey::Scores, b"{not-json".to_vec());
        let mut repository = Repository::new(storage);

        let error = repository.load_scores().expect_err("corruption reported");

        assert!(matches!(error, PersistenceError::CorruptData { .. }));
        assert_eq!(
            repository.storage().preserved_corrupt(StorageKey::Scores),
            &[b"{not-json".to_vec()]
        );
    }

    #[test]
    fn future_version_is_not_misreported_as_corruption() {
        let mut storage = MemoryByteStorage::default();
        storage.insert_raw(
            StorageKey::SystemState,
            br#"{"format_version":999,"privacy_acknowledged":true}"#.to_vec(),
        );
        let mut repository = Repository::new(storage);

        assert_eq!(
            repository
                .load_system_state()
                .expect_err("future version rejected"),
            PersistenceError::IncompatibleVersion {
                domain: SYSTEM_STATE_DOMAIN,
                found: 999,
                supported: SYSTEM_STATE_FORMAT_VERSION,
            }
        );
        assert!(
            repository
                .storage()
                .preserved_corrupt(StorageKey::SystemState)
                .is_empty()
        );
    }

    #[test]
    fn corrupt_scores_do_not_affect_settings() {
        let mut repository = InMemoryRepository::default();
        let settings = Settings {
            quiet_operation: true,
            ..Settings::default()
        };
        repository.save_settings(&settings).expect("settings save");
        repository.storage_mut().insert_raw(
            StorageKey::Scores,
            br#"{"format_version":1,"records":[{}]}"#.to_vec(),
        );

        assert!(repository.load_scores().is_err());
        assert_eq!(
            repository.load_settings().expect("settings intact"),
            settings
        );
    }

    #[test]
    fn unknown_fields_are_ignored_for_forward_compatible_v1_additions() {
        let mut storage = MemoryByteStorage::default();
        storage.insert_raw(
            StorageKey::SystemState,
            br#"{"format_version":1,"privacy_acknowledged":true,"future_hint":"ignored"}"#.to_vec(),
        );
        let mut repository = Repository::new(storage);

        assert!(
            repository
                .load_system_state()
                .expect("known v1 with extra field")
                .privacy_acknowledged
        );
    }

    #[test]
    fn reset_removes_only_the_requested_domain() {
        let mut repository = InMemoryRepository::default();
        repository
            .save_settings(&Settings::default())
            .expect("settings save");
        repository.save_scores(&[score(1)]).expect("scores save");

        repository.reset_scores().expect("score reset");

        assert!(repository.load_scores().expect("scores absent").is_empty());
        assert_eq!(
            repository.load_settings().expect("settings retained"),
            Settings::default()
        );
    }

    #[derive(Default)]
    struct FailingStorage;

    impl ByteStorage for FailingStorage {
        fn read(&self, _key: StorageKey) -> Result<Option<Vec<u8>>, StorageError> {
            Err(StorageError::new("device unavailable"))
        }

        fn write(&mut self, _key: StorageKey, _data: &[u8]) -> Result<(), StorageError> {
            Err(StorageError::new("quota exceeded"))
        }

        fn remove(&mut self, _key: StorageKey) -> Result<(), StorageError> {
            Err(StorageError::new("read only"))
        }

        fn preserve_corrupt(&mut self, _key: StorageKey, _data: &[u8]) -> Result<(), StorageError> {
            Err(StorageError::new("device unavailable"))
        }
    }

    #[test]
    fn adapter_failures_are_visible_to_the_application() {
        let mut repository = Repository::new(FailingStorage);
        assert!(matches!(
            repository.load_settings(),
            Err(PersistenceError::Unavailable(_))
        ));
        assert!(matches!(
            repository.save_scores(&[]),
            Err(PersistenceError::WriteFailed { .. })
        ));
    }
}
