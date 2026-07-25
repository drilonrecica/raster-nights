// SPDX-License-Identifier: MPL-2.0

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

/// Stable identity for a semantic UI node.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SemanticId(String);

impl SemanticId {
    pub fn parse(value: impl Into<String>) -> Result<Self, SemanticIdError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 96
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'-' | b'_' | b'.')
            });
        if valid {
            Ok(Self(value))
        } else {
            Err(SemanticIdError)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SemanticId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for SemanticId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SemanticId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("semantic ID must contain 1-96 lowercase ASCII letters, digits, '.', '_', or '-'")]
pub struct SemanticIdError;

/// Semantic role mapped to native accessibility elements by a host.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticRole {
    Application,
    Dialog,
    Heading,
    List,
    ListItem,
    Button,
    Status,
    TextInput,
    Grid,
    Row,
    GridCell,
}

/// Live-region behavior for dynamic status content.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LiveRegion {
    Polite,
    Assertive,
}

/// Optional state advertised for a semantic node.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SemanticState {
    pub focused: bool,
    pub selected: Option<bool>,
    pub disabled: bool,
    pub expanded: Option<bool>,
    pub live: Option<LiveRegion>,
}

/// Action a semantic node supports.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticActionKind {
    Activate,
    Focus,
    Increment,
    Decrement,
    SetText,
    MoveInGrid,
}

/// Direction payload for semantic grid movement.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GridDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Command received from an accessibility host.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticCommand {
    Activate,
    Focus,
    Increment,
    Decrement,
    SetText(String),
    MoveInGrid(GridDirection),
}

/// Host-independent semantic node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticNode {
    pub id: SemanticId,
    pub role: SemanticRole,
    pub label: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub state: SemanticState,
    pub actions: Vec<SemanticActionKind>,
    pub children: Vec<Self>,
}

/// Versioned semantic description of the current application screen.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticUiTree {
    pub revision: u64,
    pub root: SemanticNode,
}

/// Semantic command targeted at a stable node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticEvent {
    pub id: SemanticId,
    pub command: SemanticCommand,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_ids_accept_hierarchy_but_reject_display_text() {
        assert!(SemanticId::parse("launcher.featured.signal-stack").is_ok());
        assert!(SemanticId::parse("Signal Stack").is_err());
    }
}
