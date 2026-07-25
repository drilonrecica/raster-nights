// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

const MAX_IDENTIFIER_LENGTH: usize = 64;

/// A validated stable game identifier such as `signal-stack`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GameId(String);

impl GameId {
    pub fn parse(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated stable game-mode identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModeId(String);

impl ModeId {
    pub fn parse(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        validate_identifier(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

macro_rules! impl_string_identifier {
    ($type:ty) => {
        impl fmt::Display for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse(value).map_err(de::Error::custom)
            }
        }

        impl TryFrom<&str> for $type {
            type Error = IdentifierError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }
    };
}

impl_string_identifier!(GameId);
impl_string_identifier!(ModeId);

/// Revision of authoritative game rules.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RulesRevision(u16);

impl RulesRevision {
    pub const fn new(value: u16) -> Result<Self, IdentifierError> {
        if value == 0 {
            Err(IdentifierError::ZeroRulesRevision)
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for RulesRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u16::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Seed injected when a deterministic game run starts.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct RunSeed(pub u64);

/// Monotonic authoritative simulation tick.
#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct SimulationTick(pub u64);

impl SimulationTick {
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// One fixed authoritative simulation update.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SimulationStep {
    pub tick: SimulationTick,
}

/// Stable authoritative state hash used by golden runs.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StateHash(pub u64);

/// Lifecycle status reported by a game implementation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameStatus {
    #[default]
    Ready,
    Running,
    Finished,
}

/// Why a game run ended.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameOutcome {
    Completed,
    GameOver,
    Abandoned,
}

/// Host-independent result envelope produced by a finished game.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameResult {
    pub game_id: GameId,
    pub mode_id: ModeId,
    pub rules_revision: RulesRevision,
    pub seed: RunSeed,
    pub final_tick: SimulationTick,
    pub score: u64,
    pub outcome: GameOutcome,
    pub final_state_hash: StateHash,
}

/// Validated, normalized score-board tag.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ThreeCharacterTag([u8; 3]);

impl ThreeCharacterTag {
    pub fn parse(value: &str) -> Result<Self, IdentifierError> {
        let bytes = value.as_bytes();
        if bytes.len() != 3 {
            return Err(IdentifierError::InvalidTagLength);
        }
        if !bytes.iter().all(u8::is_ascii_alphanumeric) {
            return Err(IdentifierError::InvalidTagCharacter);
        }
        Ok(Self([
            bytes[0].to_ascii_uppercase(),
            bytes[1].to_ascii_uppercase(),
            bytes[2].to_ascii_uppercase(),
        ]))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        // SAFETY: construction accepts only ASCII bytes.
        std::str::from_utf8(&self.0).expect("tag invariant guarantees UTF-8")
    }
}

impl fmt::Display for ThreeCharacterTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for ThreeCharacterTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ThreeCharacterTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}

/// Validation error for persisted domain identifiers.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifierError {
    #[error("identifier cannot be empty")]
    Empty,
    #[error("identifier exceeds {MAX_IDENTIFIER_LENGTH} bytes")]
    TooLong,
    #[error("identifier must use lowercase ASCII letters, digits, and single hyphens")]
    InvalidCharacter,
    #[error("rules revision must be greater than zero")]
    ZeroRulesRevision,
    #[error("score tag must contain exactly three ASCII characters")]
    InvalidTagLength,
    #[error("score tag accepts only ASCII letters and digits")]
    InvalidTagCharacter,
}

fn validate_identifier(value: &str) -> Result<(), IdentifierError> {
    if value.is_empty() {
        return Err(IdentifierError::Empty);
    }
    if value.len() > MAX_IDENTIFIER_LENGTH {
        return Err(IdentifierError::TooLong);
    }

    let bytes = value.as_bytes();
    let valid = bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        && bytes.first() != Some(&b'-')
        && bytes.last() != Some(&b'-')
        && !bytes.windows(2).any(|pair| pair == b"--");
    if valid {
        Ok(())
    } else {
        Err(IdentifierError::InvalidCharacter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_accept_canonical_slugs() {
        assert_eq!(
            GameId::parse("signal-stack")
                .expect("canonical ID is valid")
                .as_str(),
            "signal-stack"
        );
        assert!(ModeId::parse("Standard Transmission").is_err());
        assert!(ModeId::parse("-standard").is_err());
        assert!(ModeId::parse("standard--transmission").is_err());
    }

    #[test]
    fn tag_is_normalized_to_uppercase() {
        let tag = ThreeCharacterTag::parse("n7l").expect("tag is valid");

        assert_eq!(tag.as_str(), "N7L");
    }

    #[test]
    fn tag_rejects_non_ascii_or_wrong_length() {
        assert!(ThreeCharacterTag::parse("AB").is_err());
        assert!(ThreeCharacterTag::parse("A_B").is_err());
        assert!(ThreeCharacterTag::parse("ÅBC").is_err());
    }

    #[test]
    fn deserialization_revalidates_identifiers() {
        let error = serde_json::from_str::<GameId>(r#""INVALID""#)
            .expect_err("invalid persisted ID must be rejected");

        assert!(error.to_string().contains("lowercase ASCII"));
    }
}
