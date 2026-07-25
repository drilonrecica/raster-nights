// SPDX-License-Identifier: MPL-2.0

//! Browser host for Raster Nights.

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::{io, rc::Rc};

    use raster_display::{
        CellStyle, DISPLAY_HEIGHT, DISPLAY_WIDTH, Display, DisplayBuffer, GridPoint, SemanticColor,
        copy_to_ratatui, render_diagnostic_grid,
    };
    use ratzilla::{
        CanvasBackend, WebEventHandler, WebGl2Backend, WebRenderer,
        backend::{canvas::CanvasBackendOptions, webgl2::WebGl2BackendOptions},
        ratatui::{Terminal, backend::Backend},
    };
    use wasm_bindgen::prelude::*;

    const DISPLAY_ELEMENT_ID: &str = "drx90-display";
    // Ratzilla's bundled static atlas uses 12x20 cells.
    const WEBGL_PIXEL_SIZE: (u32, u32) = (1_200, 720);
    const CANVAS_PIXEL_SIZE: (u32, u32) = (1_000, 684);

    /// Starts the DRX-90 browser renderer after explicit website power-on.
    #[wasm_bindgen]
    pub fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();

        let mut display = DisplayBuffer::canonical();
        render_diagnostic_grid(&mut display).map_err(js_error)?;
        display
            .text(
                GridPoint::new(3, 34),
                "HOST: BROWSER / RENDERER: AUTO",
                CellStyle::new(SemanticColor::Muted, SemanticColor::Background),
            )
            .map_err(js_error)?;
        let display = Rc::new(display);

        let webgl_options = WebGl2BackendOptions::new()
            .grid_id(DISPLAY_ELEMENT_ID)
            .size(WEBGL_PIXEL_SIZE)
            .disable_auto_css_resize();
        match WebGl2Backend::new_with_options(webgl_options) {
            Ok(backend) => run_renderer(backend, display),
            Err(_) => {
                clear_display_mount()?;
                let canvas_options = CanvasBackendOptions::new()
                    .grid_id(DISPLAY_ELEMENT_ID)
                    .size(CANVAS_PIXEL_SIZE);
                let backend = CanvasBackend::new_with_options(canvas_options).map_err(js_error)?;
                run_renderer(backend, display)
            }
        }
    }

    fn run_renderer<B>(backend: B, display: Rc<DisplayBuffer>) -> Result<(), JsValue>
    where
        B: Backend<Error = io::Error> + WebEventHandler + 'static,
    {
        let terminal = Terminal::new(backend).map_err(js_error)?;
        terminal.draw_web(move |frame| {
            frame.buffer_mut().reset();
            let area = frame.area();
            if area.width < DISPLAY_WIDTH || area.height < DISPLAY_HEIGHT {
                frame.buffer_mut().set_string(
                    area.x,
                    area.y,
                    "DRX-90 DISPLAY AREA UNAVAILABLE",
                    ratzilla::ratatui::style::Style::default()
                        .fg(ratzilla::ratatui::style::Color::Yellow),
                );
                return;
            }
            let origin = GridPoint::new(
                area.x + (area.width - DISPLAY_WIDTH) / 2,
                area.y + (area.height - DISPLAY_HEIGHT) / 2,
            );
            copy_to_ratatui(&display, frame.buffer_mut(), origin);
        });
        Ok(())
    }

    fn clear_display_mount() -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document unavailable"))?;
        let mount = document
            .get_element_by_id(DISPLAY_ELEMENT_ID)
            .ok_or_else(|| JsValue::from_str("DRX-90 display mount unavailable"))?;
        mount.set_inner_html("");
        Ok(())
    }

    fn js_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::start;
