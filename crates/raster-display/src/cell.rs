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

/// One resolved RGB color used by host renderers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }
}

/// Complete semantic-color mapping for one authored display palette.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Palette {
    colors: [RgbColor; 10],
}

impl Palette {
    #[must_use]
    pub const fn rcw_standard() -> Self {
        Self::new([
            RgbColor::new(2, 8, 10),
            RgbColor::new(10, 20, 24),
            RgbColor::new(205, 219, 214),
            RgbColor::new(103, 126, 124),
            RgbColor::new(69, 224, 211),
            RgbColor::new(76, 139, 245),
            RgbColor::new(214, 100, 255),
            RgbColor::new(98, 214, 130),
            RgbColor::new(242, 193, 76),
            RgbColor::new(238, 92, 92),
        ])
    }

    #[must_use]
    pub const fn amber_office() -> Self {
        Self::new([
            RgbColor::new(12, 8, 2),
            RgbColor::new(29, 19, 5),
            RgbColor::new(255, 211, 117),
            RgbColor::new(166, 119, 48),
            RgbColor::new(255, 190, 61),
            RgbColor::new(222, 153, 36),
            RgbColor::new(255, 225, 148),
            RgbColor::new(230, 192, 80),
            RgbColor::new(255, 173, 51),
            RgbColor::new(255, 116, 67),
        ])
    }

    #[must_use]
    pub const fn green_phosphor() -> Self {
        Self::new([
            RgbColor::new(1, 9, 4),
            RgbColor::new(4, 25, 11),
            RgbColor::new(174, 255, 191),
            RgbColor::new(74, 145, 88),
            RgbColor::new(86, 255, 126),
            RgbColor::new(75, 205, 113),
            RgbColor::new(190, 255, 171),
            RgbColor::new(116, 255, 143),
            RgbColor::new(220, 238, 106),
            RgbColor::new(255, 123, 98),
        ])
    }

    #[must_use]
    pub const fn midnight_vga() -> Self {
        Self::new([
            RgbColor::new(1, 2, 18),
            RgbColor::new(8, 11, 42),
            RgbColor::new(231, 235, 255),
            RgbColor::new(126, 137, 184),
            RgbColor::new(61, 232, 255),
            RgbColor::new(73, 108, 255),
            RgbColor::new(255, 70, 220),
            RgbColor::new(80, 238, 137),
            RgbColor::new(255, 209, 64),
            RgbColor::new(255, 72, 96),
        ])
    }

    #[must_use]
    pub const fn high_contrast() -> Self {
        Self::new([
            RgbColor::new(0, 0, 0),
            RgbColor::new(0, 0, 0),
            RgbColor::new(255, 255, 255),
            RgbColor::new(190, 190, 190),
            RgbColor::new(0, 255, 255),
            RgbColor::new(128, 200, 255),
            RgbColor::new(255, 128, 255),
            RgbColor::new(128, 255, 128),
            RgbColor::new(255, 255, 0),
            RgbColor::new(255, 96, 96),
        ])
    }

    #[must_use]
    pub const fn paper_terminal() -> Self {
        Self::new([
            RgbColor::new(242, 236, 215),
            RgbColor::new(224, 216, 190),
            RgbColor::new(25, 31, 31),
            RgbColor::new(91, 96, 91),
            RgbColor::new(0, 91, 105),
            RgbColor::new(35, 73, 145),
            RgbColor::new(113, 53, 125),
            RgbColor::new(28, 111, 64),
            RgbColor::new(143, 87, 0),
            RgbColor::new(170, 38, 38),
        ])
    }

    #[must_use]
    pub const fn resolve(self, color: SemanticColor) -> RgbColor {
        self.colors[color as usize]
    }

    const fn new(colors: [RgbColor; 10]) -> Self {
        Self { colors }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::rcw_standard()
    }
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

    #[test]
    fn high_contrast_keeps_text_and_status_roles_distinct() {
        let palette = Palette::high_contrast();
        assert_eq!(
            palette.resolve(SemanticColor::Background),
            RgbColor::new(0, 0, 0)
        );
        assert_eq!(
            palette.resolve(SemanticColor::Text),
            RgbColor::new(255, 255, 255)
        );
        assert_ne!(
            palette.resolve(SemanticColor::Warning),
            palette.resolve(SemanticColor::Danger)
        );
        assert_ne!(
            palette.resolve(SemanticColor::Primary),
            palette.resolve(SemanticColor::Text)
        );
        for role in [
            SemanticColor::Text,
            SemanticColor::Muted,
            SemanticColor::Primary,
            SemanticColor::Secondary,
            SemanticColor::Accent,
            SemanticColor::Success,
            SemanticColor::Warning,
            SemanticColor::Danger,
        ] {
            assert!(
                contrast_ratio(
                    palette.resolve(role),
                    palette.resolve(SemanticColor::Background)
                ) >= 7.0,
                "{role:?} does not meet the enhanced text contrast target"
            );
        }
    }

    fn contrast_ratio(left: RgbColor, right: RgbColor) -> f64 {
        let left = relative_luminance(left);
        let right = relative_luminance(right);
        (left.max(right) + 0.05) / (left.min(right) + 0.05)
    }

    fn relative_luminance(color: RgbColor) -> f64 {
        fn channel(value: u8) -> f64 {
            let value = f64::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(color.red) + 0.7152 * channel(color.green) + 0.0722 * channel(color.blue)
    }
}
