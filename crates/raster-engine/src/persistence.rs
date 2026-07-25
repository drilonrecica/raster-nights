// SPDX-License-Identifier: MPL-2.0

//! Host-independent persisted domain records and repository ports.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

use crate::{
    GameId, GameOutcome, ModeId, RulesRevision, RunSeed, SimulationTick, StateHash,
    ThreeCharacterTag,
};

/// User-selectable display palette.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayPalette {
    #[default]
    RcwStandard,
    AmberOffice,
    GreenPhosphor,
    MidnightVga,
    HighContrast,
    PaperTerminal,
}

/// Visual-effects preset. Individual controls remain explicit for accessibility.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffectsProfile {
    Clean,
    #[default]
    Authentic,
    Intense,
    Custom,
}

/// Stable settings the application may persist between sessions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub display_palette: DisplayPalette,
    pub effects_profile: EffectsProfile,
    pub reduced_motion: bool,
    pub reduced_flashing: bool,
    pub screen_shake: bool,
    pub crt_effects: bool,
    pub muted: bool,
    pub quiet_operation: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            display_palette: DisplayPalette::default(),
            effects_profile: EffectsProfile::default(),
            reduced_motion: false,
            reduced_flashing: false,
            screen_shake: true,
            crt_effects: true,
            muted: true,
            quiet_operation: false,
        }
    }
}

/// Stable assisted-rules profile identifier used as a score comparison dimension.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssistanceProfileId(String);

impl AssistanceProfileId {
    pub fn parse(value: impl Into<String>) -> Result<Self, AssistanceProfileIdError> {
        let value = value.into();
        let bytes = value.as_bytes();
        let valid = !bytes.is_empty()
            && bytes.len() <= 64
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
            && bytes.first() != Some(&b'-')
            && bytes.last() != Some(&b'-')
            && !bytes.windows(2).any(|pair| pair == b"--");
        if valid {
            Ok(Self(value))
        } else {
            Err(AssistanceProfileIdError)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AssistanceProfileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for AssistanceProfileId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AssistanceProfileId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("assistance profile must be a lowercase ASCII slug of at most 64 bytes")]
pub struct AssistanceProfileIdError;

/// One committed local score.
///
/// Vector order is insertion order. Consumers sort equal scores stably so an
/// older record remains ahead of a newer tie.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub game_id: GameId,
    pub mode_id: ModeId,
    pub rules_revision: RulesRevision,
    pub assistance_profile: AssistanceProfileId,
    pub tag: ThreeCharacterTag,
    pub score: u64,
    pub duration: SimulationTick,
    pub seed: RunSeed,
    pub outcome: GameOutcome,
    pub final_state_hash: StateHash,
    pub recorded_at_unix_seconds: i64,
}

impl ScoreRecord {
    #[must_use]
    pub fn is_comparable_with(&self, other: &Self) -> bool {
        self.game_id == other.game_id
            && self.mode_id == other.mode_id
            && self.rules_revision == other.rules_revision
            && self.assistance_profile == other.assistance_profile
    }
}

/// Small set of machine state remembered between sessions.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SystemState {
    pub privacy_acknowledged: bool,
    pub last_selected_game: Option<GameId>,
    pub last_game_mode: Option<ModeId>,
    pub last_score_tag: Option<ThreeCharacterTag>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PersistenceError {
    #[error("persistence is unavailable: {0}")]
    Unavailable(String),
    #[error("stored {domain} data is corrupt: {message}")]
    CorruptData {
        domain: &'static str,
        message: String,
    },
    #[error("stored {domain} format version {found} is newer than supported version {supported}")]
    IncompatibleVersion {
        domain: &'static str,
        found: u16,
        supported: u16,
    },
    #[error("could not write {domain}: {message}")]
    WriteFailed {
        domain: &'static str,
        message: String,
    },
}

pub trait SettingsRepository {
    fn load_settings(&mut self) -> Result<Settings, PersistenceError>;
    fn save_settings(&mut self, settings: &Settings) -> Result<(), PersistenceError>;
}

pub trait ScoreRepository {
    fn load_scores(&mut self) -> Result<Vec<ScoreRecord>, PersistenceError>;
    fn save_scores(&mut self, scores: &[ScoreRecord]) -> Result<(), PersistenceError>;
}

pub trait SystemStateRepository {
    fn load_system_state(&mut self) -> Result<SystemState, PersistenceError>;
    fn save_system_state(&mut self, state: &SystemState) -> Result<(), PersistenceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assistance_profile_is_a_stable_slug() {
        assert_eq!(
            AssistanceProfileId::parse("canonical")
                .expect("valid profile")
                .as_str(),
            "canonical"
        );
        assert!(AssistanceProfileId::parse("Canonical").is_err());
        assert!(AssistanceProfileId::parse("two--hyphens").is_err());
    }

    #[test]
    fn settings_defaults_keep_browser_audio_muted() {
        let settings = Settings::default();
        assert!(settings.muted);
        assert!(!settings.reduced_motion);
    }
}
