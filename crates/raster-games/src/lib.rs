// SPDX-License-Identifier: MPL-2.0

//! Official game implementations and their explicit registry.

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use raster_display::{DISPLAY_HEIGHT, DISPLAY_WIDTH, DisplayBuffer, render_diagnostic_grid};
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn shared_display_composition_runs_in_wasm() {
        let mut display = DisplayBuffer::canonical();
        render_diagnostic_grid(&mut display).expect("diagnostic grid must be valid");

        assert_eq!(display.snapshot().size.width, DISPLAY_WIDTH);
        assert_eq!(display.snapshot().size.height, DISPLAY_HEIGHT);
    }
}
