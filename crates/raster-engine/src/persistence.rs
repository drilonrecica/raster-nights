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

pub const LOCAL_SCORE_LIMIT: usize = 10;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreRankingKey {
    pub game_id: GameId,
    pub mode_id: ModeId,
    pub rules_revision: RulesRevision,
    pub assistance_profile: AssistanceProfileId,
}

impl ScoreRankingKey {
    #[must_use]
    pub fn matches(&self, record: &ScoreRecord) -> bool {
        self.game_id == record.game_id
            && self.mode_id == record.mode_id
            && self.rules_revision == record.rules_revision
            && self.assistance_profile == record.assistance_profile
    }
}

impl ScoreRecord {
    #[must_use]
    pub fn is_comparable_with(&self, other: &Self) -> bool {
        self.game_id == other.game_id
            && self.mode_id == other.mode_id
            && self.rules_revision == other.rules_revision
            && self.assistance_profile == other.assistance_profile
    }

    #[must_use]
    pub fn ranking_key(&self) -> ScoreRankingKey {
        ScoreRankingKey {
            game_id: self.game_id.clone(),
            mode_id: self.mode_id.clone(),
            rules_revision: self.rules_revision,
            assistance_profile: self.assistance_profile.clone(),
        }
    }
}

#[must_use]
pub fn ranked_scores<'a>(scores: &'a [ScoreRecord], key: &ScoreRankingKey) -> Vec<&'a ScoreRecord> {
    let mut ranked = scores
        .iter()
        .filter(|record| key.matches(record))
        .collect::<Vec<_>>();
    ranked.sort_by_key(|record| std::cmp::Reverse(record.score));
    ranked.truncate(LOCAL_SCORE_LIMIT);
    ranked
}

#[must_use]
pub fn score_qualifies(scores: &[ScoreRecord], key: &ScoreRankingKey, score: u64) -> bool {
    let ranked = ranked_scores(scores, key);
    ranked.len() < LOCAL_SCORE_LIMIT || ranked.last().is_some_and(|cutoff| score > cutoff.score)
}

/// Inserts a qualifying score and enforces the per-ranking-key limit.
///
/// Equal scores retain insertion order and never displace an older cutoff.
pub fn insert_score(scores: &mut Vec<ScoreRecord>, record: ScoreRecord) -> bool {
    let key = record.ranking_key();
    if !score_qualifies(scores, &key, record.score) {
        return false;
    }
    scores.push(record);

    let comparable_count = scores.iter().filter(|entry| key.matches(entry)).count();
    if comparable_count <= LOCAL_SCORE_LIMIT {
        return true;
    }

    let worst_index = scores
        .iter()
        .enumerate()
        .filter(|(_, entry)| key.matches(entry))
        .reduce(|worst, candidate| {
            if candidate.1.score < worst.1.score
                || (candidate.1.score == worst.1.score && candidate.0 > worst.0)
            {
                candidate
            } else {
                worst
            }
        })
        .map(|(index, _)| index)
        .expect("a count above the limit guarantees a comparable score");
    scores.remove(worst_index);
    true
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

    fn score(value: u64, recorded_at: i64) -> ScoreRecord {
        ScoreRecord {
            game_id: GameId::parse("signal-stack").expect("valid ID"),
            mode_id: ModeId::parse("standard-transmission").expect("valid ID"),
            rules_revision: RulesRevision::new(1).expect("valid revision"),
            assistance_profile: AssistanceProfileId::parse("canonical").expect("valid profile"),
            tag: ThreeCharacterTag::parse("NUL").expect("valid tag"),
            score: value,
            duration: SimulationTick(60),
            seed: RunSeed(1),
            outcome: GameOutcome::GameOver,
            final_state_hash: StateHash(value),
            recorded_at_unix_seconds: recorded_at,
        }
    }

    #[test]
    fn ranking_is_descending_and_stable_for_ties() {
        let scores = vec![score(100, 1), score(200, 2), score(100, 3)];
        let ranked = ranked_scores(&scores, &scores[0].ranking_key());

        assert_eq!(
            ranked
                .iter()
                .map(|record| record.recorded_at_unix_seconds)
                .collect::<Vec<_>>(),
            vec![2, 1, 3]
        );
    }

    #[test]
    fn equal_cutoff_does_not_displace_older_record() {
        let mut scores = (0..LOCAL_SCORE_LIMIT)
            .map(|index| score(1_000 - index as u64, index as i64))
            .collect::<Vec<_>>();
        let cutoff = scores.last().expect("ten scores").score;

        assert!(!insert_score(&mut scores, score(cutoff, 99)));
        assert_eq!(scores.len(), LOCAL_SCORE_LIMIT);
        assert!(
            scores
                .iter()
                .all(|record| record.recorded_at_unix_seconds != 99)
        );
    }

    #[test]
    fn higher_score_displaces_only_newest_lowest_record() {
        let mut scores = (0..LOCAL_SCORE_LIMIT)
            .map(|index| score(100, index as i64))
            .collect::<Vec<_>>();

        assert!(insert_score(&mut scores, score(101, 99)));
        assert_eq!(scores.len(), LOCAL_SCORE_LIMIT);
        assert!(
            scores
                .iter()
                .any(|record| record.recorded_at_unix_seconds == 0)
        );
        assert!(
            scores
                .iter()
                .all(|record| record.recorded_at_unix_seconds != 9)
        );
    }
}
