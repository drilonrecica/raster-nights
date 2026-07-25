// SPDX-License-Identifier: MPL-2.0

//! Official game implementations and their explicit registry.

pub mod loopback;
pub mod packet_sweep;
mod registry;
pub mod signal_stack;

pub use registry::{CatalogError, RasterGameRegistry};

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use crate::signal_stack::SignalStack;
    use raster_display::{DISPLAY_HEIGHT, DISPLAY_WIDTH, DisplayBuffer, render_diagnostic_grid};
    use raster_engine::{
        ActionPhase, AppAction, AppStateKind, Application, CalendarDate, GameAction, HostKind,
        RunSeed, SimulationStep, SimulationTick, StateHash,
    };
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn shared_display_composition_runs_in_wasm() {
        let mut display = DisplayBuffer::canonical();
        render_diagnostic_grid(&mut display).expect("diagnostic grid must be valid");

        assert_eq!(display.snapshot().size.width, DISPLAY_WIDTH);
        assert_eq!(display.snapshot().size.height, DISPLAY_HEIGHT);
    }

    #[wasm_bindgen_test]
    fn shared_system_shell_runs_in_browser_wasm() {
        let mut app = Application::new(HostKind::Browser, CalendarDate::new(25, 7, 2026), false);
        let mut display = DisplayBuffer::canonical();

        app.render(&mut display)
            .expect("privacy notice should compose in Wasm");
        app.handle_action(AppAction::Confirm, ActionPhase::Pressed);
        app.handle_action(AppAction::Confirm, ActionPhase::Pressed);
        app.render(&mut display)
            .expect("launcher should compose in Wasm");

        assert_eq!(app.state_kind(), AppStateKind::Launcher);
        assert!(display.snapshot().character_grid().contains("SIGNAL STACK"));
    }

    #[wasm_bindgen_test]
    fn signal_stack_golden_run_matches_native_expectation() {
        let mut game = SignalStack::new(RunSeed(0xCAFE_BABE));
        for tick in 1..=180 {
            let actions: &[GameAction] = match tick {
                3 | 5 | 7 => &[GameAction::MoveLeft],
                11 => &[GameAction::RotateClockwise],
                15 => &[GameAction::Hold],
                24 | 48 | 72 => &[GameAction::SoftDrop],
                90 => &[GameAction::HardDrop],
                100 => &[GameAction::MoveRight, GameAction::RotateCounterclockwise],
                150 => &[GameAction::HardDrop],
                _ => &[],
            };
            for action in actions {
                game.handle_action(*action);
            }
            game.update(SimulationStep {
                tick: SimulationTick(tick),
            });
        }

        assert_eq!(game.state_hash(), StateHash(14_724_018_137_410_630_377));
    }
}
