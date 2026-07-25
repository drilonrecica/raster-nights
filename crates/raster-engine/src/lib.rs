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
    DiscoveryMarker, GameId, GameOutcome, GameResult, GameStatus, IdentifierError, ModeId,
    RulesRevision, RunSeed, SimulationStep, SimulationTick, StateHash, ThreeCharacterTag,
};
pub use game::{
    CatalogVisibility, ControlDescription, DirectLaunchRequest, Game, GameCategory, GameDescriptor,
    GameError, GameRegistry, ModeDescriptor, NewRunRequest, RunMetadataSource, StartupOptions,
};
pub use input::{
    AppAction, DeviceInput, GameAction, InputCapability, InputContext, KeyCode, KeyModifiers,
    PhysicalKey, PointerButton, TextEscapeBehavior,
};
pub use input_system::{ActionEvent, ActionPhase, InputSystem, RepeatProfile, map_key_to_action};
pub use persistence::{
    ApplicationRepository, AssistanceProfileId, DisplayPalette, EffectsProfile, LOCAL_SCORE_LIMIT,
    PersistenceError, ScoreRankingKey, ScoreRecord, ScoreRepository, Settings, SettingsRepository,
    SystemState, SystemStateRepository, insert_score, normalize_scores, ranked_scores,
    score_qualifies,
};
pub use semantic::{
    GridDirection, LiveRegion, SemanticActionKind, SemanticCommand, SemanticEvent, SemanticId,
    SemanticNode, SemanticRole, SemanticState, SemanticUiTree,
};
