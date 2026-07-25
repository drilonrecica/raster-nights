// SPDX-License-Identifier: MPL-2.0

use std::fmt::Debug;

use raster_display::{Display, GlyphError, GridSize};
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameDescriptor {
    pub id: GameId,
    pub title: &'static str,
    pub category: &'static str,
    pub fictional_release_date: &'static str,
    pub fictional_developer: &'static str,
    pub fictional_publisher: &'static str,
    pub premise: &'static str,
    pub rules_revision: RulesRevision,
    pub minimum_grid: GridSize,
    pub modes: Vec<ModeDescriptor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeDescriptor {
    pub id: ModeId,
    pub title: &'static str,
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
    fn descriptors(&self) -> Vec<GameDescriptor>;
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
    #[error("game failed: {0}")]
    Runtime(String),
}
