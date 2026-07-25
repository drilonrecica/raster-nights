// SPDX-License-Identifier: MPL-2.0

use thiserror::Error;
use unicode_width::UnicodeWidthChar;

/// A semantic color resolved by each host theme.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SemanticColor {
    #[default]
    Background,
    Surface,
    Text,
    Muted,
    Primary,
    Secondary,
    Accent,
    Success,
    Warning,
    Danger,
}

/// Cell presentation modifiers.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CellModifiers {
    pub bold: bool,
    pub dim: bool,
    pub underlined: bool,
    pub reversed: bool,
}

/// Complete presentation style for a display cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CellStyle {
    pub foreground: SemanticColor,
    pub background: SemanticColor,
    pub modifiers: CellModifiers,
}

impl CellStyle {
    #[must_use]
    pub const fn new(foreground: SemanticColor, background: SemanticColor) -> Self {
        Self {
            foreground,
            background,
            modifiers: CellModifiers {
                bold: false,
                dim: false,
                underlined: false,
                reversed: false,
            },
        }
    }

    #[must_use]
    pub const fn bold(mut self) -> Self {
        self.modifiers.bold = true;
        self
    }
}

impl Default for CellStyle {
    fn default() -> Self {
        Self::new(SemanticColor::Text, SemanticColor::Background)
    }
}

/// Style applied while drawing text.
pub type TextStyle = CellStyle;

/// A validated single-width cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GameCell {
    glyph: char,
    pub style: CellStyle,
}

impl GameCell {
    pub fn new(glyph: char, style: CellStyle) -> Result<Self, GlyphError> {
        validate_glyph(glyph)?;
        Ok(Self { glyph, style })
    }

    #[must_use]
    pub const fn space(style: CellStyle) -> Self {
        Self { glyph: ' ', style }
    }

    #[must_use]
    pub const fn glyph(self) -> char {
        self.glyph
    }
}

impl Default for GameCell {
    fn default() -> Self {
        Self::space(CellStyle::default())
    }
}

/// Glyphs used to draw a rectangular border.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BorderGlyphs {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl BorderGlyphs {
    pub const ASCII: Self = Self {
        top_left: '+',
        top_right: '+',
        bottom_left: '+',
        bottom_right: '+',
        horizontal: '-',
        vertical: '|',
    };

    pub fn validate(self) -> Result<(), GlyphError> {
        for glyph in [
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
            self.horizontal,
            self.vertical,
        ] {
            validate_glyph(glyph)?;
        }
        Ok(())
    }
}

/// Border glyphs and style.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BorderStyle {
    pub glyphs: BorderGlyphs,
    pub cell_style: CellStyle,
}

impl BorderStyle {
    #[must_use]
    pub const fn ascii(cell_style: CellStyle) -> Self {
        Self {
            glyphs: BorderGlyphs::ASCII,
            cell_style,
        }
    }
}

/// An invalid canonical display glyph.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("glyph {glyph:?} has display width {width:?}; expected exactly one cell")]
pub struct GlyphError {
    pub glyph: char,
    pub width: Option<usize>,
}

/// Ensures a glyph occupies exactly one terminal cell.
pub fn validate_glyph(glyph: char) -> Result<(), GlyphError> {
    let width = glyph.width();
    if width == Some(1) {
        Ok(())
    } else {
        Err(GlyphError { glyph, width })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_single_cell_ascii() {
        assert_eq!(validate_glyph('A'), Ok(()));
        assert_eq!(validate_glyph(' '), Ok(()));
    }

    #[test]
    fn rejects_zero_and_double_width_glyphs() {
        assert!(validate_glyph('\u{0301}').is_err());
        assert!(validate_glyph('界').is_err());
        assert!(validate_glyph('\n').is_err());
    }
}
