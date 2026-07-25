// SPDX-License-Identifier: MPL-2.0

//! Host-independent application state and runtime services.

mod app;
mod clock;
mod domain;
mod game;
mod input;
mod input_system;
mod persistence;
mod screens;
mod semantic;

pub use app::{AppStateKind, Application, CalendarDate, HostKind, MINIMUM_COLUMNS, MINIMUM_ROWS};
pub use clock::{FixedStepClock, MAX_FRAME_DELTA, SIMULATION_HZ, StepBatch};
pub use domain::{
    GameId, GameOutcome, GameResult, GameStatus, IdentifierError, ModeId, RulesRevision, RunSeed,
    SimulationStep, SimulationTick, StateHash, ThreeCharacterTag,
};
pub use game::{Game, GameDescriptor, GameError, GameRegistry, ModeDescriptor, NewRunRequest};
pub use input::{
    AppAction, DeviceInput, GameAction, InputCapability, InputContext, KeyCode, KeyModifiers,
    PhysicalKey, PointerButton, TextEscapeBehavior,
};
pub use input_system::{ActionEvent, ActionPhase, InputSystem, RepeatProfile, map_key_to_action};
pub use persistence::{
    AssistanceProfileId, DisplayPalette, EffectsProfile, PersistenceError, ScoreRecord,
    ScoreRepository, Settings, SettingsRepository, SystemState, SystemStateRepository,
};
pub use semantic::{
    GridDirection, LiveRegion, SemanticActionKind, SemanticCommand, SemanticEvent, SemanticId,
    SemanticNode, SemanticRole, SemanticState, SemanticUiTree,
};
