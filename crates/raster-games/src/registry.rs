// SPDX-License-Identifier: MPL-2.0

use raster_display::{DISPLAY_SIZE, Display, GlyphError};
use raster_engine::{
    ActionPhase, Game, GameDescriptor, GameError, GameId, GameRegistry, GameResult, GameStatus,
    ModeDescriptor, ModeId, NewRunRequest, RunSeed, SimulationStep,
};

use crate::signal_stack::{SignalStack, render};

#[derive(Debug, Default)]
pub struct RasterGameRegistry;

impl RasterGameRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl GameRegistry for RasterGameRegistry {
    fn descriptors(&self) -> Vec<GameDescriptor> {
        vec![signal_stack_descriptor()]
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
            descriptor: signal_stack_descriptor(),
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

fn signal_stack_descriptor() -> GameDescriptor {
    GameDescriptor {
        id: GameId::parse("signal-stack").expect("static game ID is valid"),
        title: "Signal Stack",
        category: "Puzzle",
        fictional_release_date: "21.11.1995",
        fictional_developer: "Frankenberg Logic Bureau",
        fictional_publisher: "Sara Circuitworks",
        premise: "Route falling data packets through a saturated switching matrix.",
        rules_revision: crate::signal_stack::rules_revision(),
        minimum_grid: DISPLAY_SIZE,
        modes: vec![ModeDescriptor {
            id: ModeId::parse("standard-transmission").expect("static mode ID is valid"),
            title: "Standard Transmission",
        }],
    }
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
        let descriptors = registry.descriptors();

        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].id.as_str(), "signal-stack");
        assert_eq!(descriptors[0].modes.len(), 1);
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
