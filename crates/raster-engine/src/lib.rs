// SPDX-License-Identifier: MPL-2.0

//! Host-independent application state and runtime services.

mod domain;
mod input;
mod semantic;

pub use domain::{
    GameId, GameOutcome, GameResult, GameStatus, IdentifierError, ModeId, RulesRevision, RunSeed,
    SimulationStep, SimulationTick, StateHash, ThreeCharacterTag,
};
pub use input::{
    AppAction, DeviceInput, GameAction, InputCapability, KeyCode, KeyModifiers, PhysicalKey,
    PointerButton,
};
pub use semantic::{
    GridDirection, LiveRegion, SemanticActionKind, SemanticCommand, SemanticEvent, SemanticId,
    SemanticNode, SemanticRole, SemanticState, SemanticUiTree,
};
