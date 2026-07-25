// SPDX-License-Identifier: MPL-2.0

//! Host-independent display types and drawing operations.

mod buffer;
mod cell;
mod diagnostics;
mod geometry;

pub use buffer::{Display, DisplayBuffer, DisplaySnapshot, DisplayViewport};
pub use cell::{
    BorderGlyphs, BorderStyle, CellModifiers, CellStyle, GameCell, GlyphError, SemanticColor,
    TextStyle, validate_glyph,
};
pub use diagnostics::render_diagnostic_grid;
pub use geometry::{GridPoint, GridRect, GridSize};

/// Width of the canonical DRX-90 logical display.
pub const DISPLAY_WIDTH: u16 = 100;

/// Height of the canonical DRX-90 logical display.
pub const DISPLAY_HEIGHT: u16 = 36;

/// Canonical DRX-90 logical display size.
pub const DISPLAY_SIZE: GridSize = GridSize::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);
