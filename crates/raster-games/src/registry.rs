// SPDX-License-Identifier: MPL-2.0

use std::collections::HashSet;

use raster_display::{DISPLAY_SIZE, Display, GlyphError};
use raster_engine::{
    ActionPhase, CatalogVisibility, ControlDescription, Game, GameCategory, GameDescriptor,
    GameError, GameId, GameRegistry, GameResult, GameStatus, ModeDescriptor, ModeId, NewRunRequest,
    RulesRevision, RunSeed, SimulationStep,
};
use serde::Deserialize;
use thiserror::Error;

use crate::signal_stack::{SignalStack, render};

const CATALOG_JSON: &str = include_str!("../../../content/catalog.json");
const CATALOG_FORMAT_VERSION: u16 = 1;

#[derive(Debug)]
pub struct RasterGameRegistry {
    advertised: Vec<GameDescriptor>,
    hidden: Vec<GameDescriptor>,
}

impl RasterGameRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::load().expect("bundled catalog is validated by repository checks")
    }

    pub fn load() -> Result<Self, CatalogError> {
        Self::from_json(CATALOG_JSON)
    }

    pub fn validate_bundled_content() -> Result<(), CatalogError> {
        Self::load().map(|_| ())
    }

    fn from_json(json: &str) -> Result<Self, CatalogError> {
        let catalog: CatalogFile = serde_json::from_str(json)
            .map_err(|error| CatalogError::InvalidJson(error.to_string()))?;
        if catalog.format_version != CATALOG_FORMAT_VERSION {
            return Err(CatalogError::UnsupportedVersion {
                found: catalog.format_version,
                supported: CATALOG_FORMAT_VERSION,
            });
        }
        if catalog.games.is_empty() {
            return Err(CatalogError::EmptyCatalog);
        }

        let mut ids = HashSet::new();
        let mut advertised = Vec::new();
        let mut hidden = Vec::new();
        for game in catalog.games {
            let descriptor = game.try_into_descriptor()?;
            if !ids.insert(descriptor.id.clone()) {
                return Err(CatalogError::DuplicateGame(descriptor.id));
            }
            match descriptor.visibility {
                CatalogVisibility::Advertised => advertised.push(descriptor),
                CatalogVisibility::Hidden => hidden.push(descriptor),
            }
        }
        validate_compiled_registration(&advertised, &hidden)?;
        Ok(Self { advertised, hidden })
    }
}

impl Default for RasterGameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRegistry for RasterGameRegistry {
    fn advertised_descriptors(&self) -> Vec<GameDescriptor> {
        self.advertised.clone()
    }

    fn hidden_descriptors(&self) -> Vec<GameDescriptor> {
        self.hidden.clone()
    }

    fn create(&self, game_id: &GameId) -> Result<Box<dyn Game>, GameError> {
        if game_id.as_str() != "signal-stack" {
            return Err(GameError::NotRegistered(game_id.clone()));
        }
        Ok(Box::new(SignalStackGame::new()))
    }
}

#[derive(Debug)]
struct SignalStackGame {
    descriptor: GameDescriptor,
    simulation: SignalStack,
}

impl SignalStackGame {
    fn new() -> Self {
        Self {
            descriptor: bundled_descriptor("signal-stack"),
            simulation: SignalStack::new(RunSeed(0)),
        }
    }
}

impl Game for SignalStackGame {
    fn descriptor(&self) -> &GameDescriptor {
        &self.descriptor
    }

    fn reset(&mut self, request: &NewRunRequest) -> Result<(), GameError> {
        if request.game_id != self.descriptor.id
            || request.mode_id.as_str() != "standard-transmission"
            || request.rules_revision != self.descriptor.rules_revision
        {
            return Err(GameError::InvalidRunRequest);
        }
        self.simulation.reset(request.seed);
        Ok(())
    }

    fn handle_action(
        &mut self,
        action: raster_engine::GameAction,
        phase: ActionPhase,
    ) -> Result<(), GameError> {
        if phase != ActionPhase::Released {
            self.simulation.handle_action(action);
        }
        Ok(())
    }

    fn update(&mut self, step: SimulationStep) -> Result<(), GameError> {
        self.simulation.update(step);
        Ok(())
    }

    fn render(&self, display: &mut dyn Display) -> Result<(), GlyphError> {
        render(&self.simulation, display)
    }

    fn set_paused(&mut self, paused: bool) {
        self.simulation.set_paused(paused);
    }

    fn status(&self) -> GameStatus {
        self.simulation.game_status()
    }

    fn result(&self) -> Option<GameResult> {
        self.simulation.result()
    }
}

fn bundled_descriptor(id: &str) -> GameDescriptor {
    let registry = RasterGameRegistry::new();
    registry
        .advertised
        .into_iter()
        .chain(registry.hidden)
        .find(|descriptor| descriptor.id.as_str() == id)
        .expect("compiled games have validated bundled descriptors")
}

fn validate_compiled_registration(
    advertised: &[GameDescriptor],
    hidden: &[GameDescriptor],
) -> Result<(), CatalogError> {
    let descriptors = advertised.iter().chain(hidden);
    let signal_stack = descriptors
        .into_iter()
        .find(|descriptor| descriptor.id.as_str() == "signal-stack")
        .ok_or_else(|| CatalogError::MissingCompiledGame("signal-stack".to_owned()))?;
    if signal_stack.rules_revision != crate::signal_stack::rules_revision() {
        return Err(CatalogError::RulesRevisionMismatch {
            game_id: signal_stack.id.clone(),
            content: signal_stack.rules_revision,
            compiled: crate::signal_stack::rules_revision(),
        });
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    format_version: u16,
    games: Vec<GameContent>,
}

#[derive(Debug, Deserialize)]
struct GameContent {
    id: GameId,
    title: String,
    short_title: String,
    category: GameCategory,
    fictional_release_date: Option<String>,
    fictional_version: String,
    catalog_number: Option<String>,
    fictional_developer: String,
    fictional_publisher: String,
    premise: String,
    visibility: CatalogVisibility,
    rules_revision: RulesRevision,
    modes: Vec<ModeContent>,
    controls: Vec<ControlContent>,
}

impl GameContent {
    fn try_into_descriptor(self) -> Result<GameDescriptor, CatalogError> {
        require_text(&self.id, "title", &self.title)?;
        require_text(&self.id, "short_title", &self.short_title)?;
        require_text(&self.id, "fictional_version", &self.fictional_version)?;
        require_text(&self.id, "fictional_developer", &self.fictional_developer)?;
        require_text(&self.id, "fictional_publisher", &self.fictional_publisher)?;
        require_text(&self.id, "premise", &self.premise)?;
        if let Some(date) = &self.fictional_release_date {
            validate_fictional_date(&self.id, date)?;
        }
        if self.modes.is_empty() {
            return Err(CatalogError::MissingModes(self.id));
        }
        if self.controls.is_empty() {
            return Err(CatalogError::MissingControls(self.id));
        }
        let mut mode_ids = HashSet::new();
        let modes = self
            .modes
            .into_iter()
            .map(|mode| {
                if !mode_ids.insert(mode.id.clone()) {
                    return Err(CatalogError::DuplicateMode {
                        game_id: self.id.clone(),
                        mode_id: mode.id,
                    });
                }
                require_text(&self.id, "mode title", &mode.title)?;
                Ok(ModeDescriptor {
                    id: mode.id,
                    title: mode.title,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let controls = self
            .controls
            .into_iter()
            .map(|control| {
                require_text(&self.id, "control label", &control.label)?;
                if control.default_bindings.is_empty()
                    || control
                        .default_bindings
                        .iter()
                        .any(|binding| binding.trim().is_empty())
                {
                    return Err(CatalogError::InvalidControl(self.id.clone()));
                }
                Ok(ControlDescription {
                    action: control.action,
                    label: control.label,
                    default_bindings: control.default_bindings,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(GameDescriptor {
            id: self.id,
            title: self.title,
            short_title: self.short_title,
            category: self.category,
            fictional_release_date: self.fictional_release_date,
            fictional_version: self.fictional_version,
            catalog_number: self.catalog_number,
            fictional_developer: self.fictional_developer,
            fictional_publisher: self.fictional_publisher,
            premise: self.premise,
            visibility: self.visibility,
            rules_revision: self.rules_revision,
            minimum_grid: DISPLAY_SIZE,
            modes,
            controls,
        })
    }
}

#[derive(Debug, Deserialize)]
struct ModeContent {
    id: ModeId,
    title: String,
}

#[derive(Debug, Deserialize)]
struct ControlContent {
    action: raster_engine::GameAction,
    label: String,
    default_bindings: Vec<String>,
}

fn require_text(game_id: &GameId, field: &'static str, value: &str) -> Result<(), CatalogError> {
    if value.trim().is_empty() {
        Err(CatalogError::MissingText {
            game_id: game_id.clone(),
            field,
        })
    } else {
        Ok(())
    }
}

fn validate_fictional_date(game_id: &GameId, date: &str) -> Result<(), CatalogError> {
    let bytes = date.as_bytes();
    if bytes.len() != 10 || bytes[2] != b'.' || bytes[5] != b'.' {
        return Err(CatalogError::InvalidDate {
            game_id: game_id.clone(),
            date: date.to_owned(),
        });
    }
    let day = date[0..2].parse::<u8>();
    let month = date[3..5].parse::<u8>();
    let year = date[6..10].parse::<u16>();
    if !matches!(
        (day, month, year),
        (Ok(1..=31), Ok(1..=12), Ok(1993..=1999))
    ) {
        return Err(CatalogError::InvalidDate {
            game_id: game_id.clone(),
            date: date.to_owned(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CatalogError {
    #[error("bundled catalog JSON is invalid: {0}")]
    InvalidJson(String),
    #[error("bundled catalog format version {found} is unsupported; expected {supported}")]
    UnsupportedVersion { found: u16, supported: u16 },
    #[error("bundled catalog contains no games")]
    EmptyCatalog,
    #[error("bundled catalog contains duplicate game ID {0}")]
    DuplicateGame(GameId),
    #[error("bundled catalog is missing compiled game {0}")]
    MissingCompiledGame(String),
    #[error(
        "bundled catalog rules revision for {game_id} is {content:?}, compiled revision is {compiled:?}"
    )]
    RulesRevisionMismatch {
        game_id: GameId,
        content: RulesRevision,
        compiled: RulesRevision,
    },
    #[error("catalog game {game_id} has empty {field}")]
    MissingText {
        game_id: GameId,
        field: &'static str,
    },
    #[error("catalog game {game_id} has invalid fictional date {date}")]
    InvalidDate { game_id: GameId, date: String },
    #[error("catalog game {0} has no modes")]
    MissingModes(GameId),
    #[error("catalog game {game_id} repeats mode {mode_id}")]
    DuplicateMode { game_id: GameId, mode_id: ModeId },
    #[error("catalog game {0} has no controls")]
    MissingControls(GameId),
    #[error("catalog game {0} has an invalid control binding")]
    InvalidControl(GameId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use raster_display::DisplayBuffer;
    use raster_engine::{GameAction, RulesRevision};

    fn request(seed: u64) -> NewRunRequest {
        NewRunRequest {
            game_id: GameId::parse("signal-stack").expect("valid ID"),
            mode_id: ModeId::parse("standard-transmission").expect("valid ID"),
            rules_revision: RulesRevision::new(1).expect("valid revision"),
            seed: RunSeed(seed),
        }
    }

    #[test]
    fn registry_exposes_only_the_installed_game() {
        let registry = RasterGameRegistry::new();
        let descriptors = registry.advertised_descriptors();

        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].id.as_str(), "signal-stack");
        assert_eq!(descriptors[0].modes.len(), 1);
        assert!(registry.hidden_descriptors().is_empty());
    }

    #[test]
    fn catalog_validation_rejects_duplicate_ids_and_bad_dates() {
        let duplicate = CATALOG_JSON.replace(
            "\"games\": [",
            "\"games\": [{\"id\":\"signal-stack\",\"title\":\"X\",\"short_title\":\"X\",\"category\":\"puzzle\",\"fictional_release_date\":\"21.11.1995\",\"fictional_version\":\"1\",\"catalog_number\":null,\"fictional_developer\":\"X\",\"fictional_publisher\":\"X\",\"premise\":\"X\",\"visibility\":\"advertised\",\"rules_revision\":1,\"modes\":[{\"id\":\"standard-transmission\",\"title\":\"X\"}],\"controls\":[{\"action\":\"primary\",\"label\":\"X\",\"default_bindings\":[\"X\"]}]},",
        );
        assert!(matches!(
            RasterGameRegistry::from_json(&duplicate),
            Err(CatalogError::DuplicateGame(_))
        ));

        let bad_date = CATALOG_JSON.replace("21.11.1995", "21-11-2005");
        assert!(matches!(
            RasterGameRegistry::from_json(&bad_date),
            Err(CatalogError::InvalidDate { .. })
        ));
    }

    #[test]
    fn lifecycle_adapter_resets_updates_and_renders() {
        let registry = RasterGameRegistry::new();
        let mut game = registry
            .create(&request(7).game_id)
            .expect("registered game");
        game.reset(&request(7)).expect("valid request");
        game.handle_action(GameAction::HardDrop, ActionPhase::Pressed)
            .expect("action");
        game.update(SimulationStep {
            tick: raster_engine::SimulationTick(1),
        })
        .expect("update");

        let mut display = DisplayBuffer::canonical();
        game.render(&mut display).expect("render");
        let grid = display.snapshot().character_grid();
        assert!(grid.contains("SIGNAL STACK"));
        assert!(grid.contains("SCORE"));
    }
}
