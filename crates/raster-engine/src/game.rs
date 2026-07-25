// SPDX-License-Identifier: MPL-2.0

use std::fmt::Debug;

use raster_display::{Display, GlyphError, GridSize};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ActionPhase, GameAction, GameId, GameResult, GameStatus, ModeId, RulesRevision, RunSeed,
    SimulationStep,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewRunRequest {
    pub game_id: GameId,
    pub mode_id: ModeId,
    pub rules_revision: RulesRevision,
    pub seed: RunSeed,
}

/// Optional native startup behavior supplied by a host command line.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StartupOptions {
    pub quiet: bool,
    pub direct_launch: Option<DirectLaunchRequest>,
}

/// Validated intent to launch one compiled game without ordinary catalog navigation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectLaunchRequest {
    pub game_id: GameId,
    pub quick: bool,
    pub seed: Option<RunSeed>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameDescriptor {
    pub id: GameId,
    pub title: String,
    pub short_title: String,
    pub category: GameCategory,
    pub fictional_release_date: Option<String>,
    pub fictional_version: String,
    pub catalog_number: Option<String>,
    pub fictional_developer: String,
    pub fictional_publisher: String,
    pub premise: String,
    pub visibility: CatalogVisibility,
    pub rules_revision: RulesRevision,
    pub minimum_grid: GridSize,
    pub modes: Vec<ModeDescriptor>,
    pub controls: Vec<ControlDescription>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDescriptor {
    pub id: ModeId,
    pub title: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameCategory {
    Puzzle,
    GridArcade,
    Action,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogVisibility {
    Advertised,
    Hidden,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlDescription {
    pub action: GameAction,
    pub label: String,
    pub default_bindings: Vec<String>,
}

pub trait Game: Debug {
    fn descriptor(&self) -> &GameDescriptor;
    fn reset(&mut self, request: &NewRunRequest) -> Result<(), GameError>;
    fn handle_action(&mut self, action: GameAction, phase: ActionPhase) -> Result<(), GameError>;
    fn update(&mut self, step: SimulationStep) -> Result<(), GameError>;
    fn render(&self, display: &mut dyn Display) -> Result<(), GlyphError>;
    fn set_paused(&mut self, paused: bool);
    fn status(&self) -> GameStatus;
    fn result(&self) -> Option<GameResult>;
}

pub trait GameRegistry: Debug {
    fn advertised_descriptors(&self) -> Vec<GameDescriptor>;
    fn hidden_descriptors(&self) -> Vec<GameDescriptor>;
    fn create(&self, game_id: &GameId) -> Result<Box<dyn Game>, GameError>;
}

/// Host-provided nondeterministic metadata used only to begin and record runs.
///
/// Games receive the resulting seed and timestamp as ordinary values and never
/// access a platform clock or entropy source directly.
pub trait RunMetadataSource: Debug {
    fn next_seed(&mut self) -> RunSeed;
    fn unix_seconds(&self) -> i64;
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum GameError {
    #[error("game {0} is not registered")]
    NotRegistered(GameId),
    #[error("run request does not match the selected game or mode")]
    InvalidRunRequest,
    #[error("game {0} is hidden and has not been unlocked")]
    Locked(GameId),
    #[error("game failed: {0}")]
    Runtime(String),
}
