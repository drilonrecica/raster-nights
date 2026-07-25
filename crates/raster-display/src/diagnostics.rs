// SPDX-License-Identifier: MPL-2.0

use crate::{
    BorderStyle, CellStyle, DISPLAY_HEIGHT, DISPLAY_WIDTH, Display, GlyphError, GridPoint,
    GridRect, SemanticColor,
};

/// Composes the canonical development display used to verify host parity.
pub fn render_diagnostic_grid(display: &mut dyn Display) -> Result<(), GlyphError> {
    let background = CellStyle::new(SemanticColor::Text, SemanticColor::Background);
    let primary = CellStyle::new(SemanticColor::Primary, SemanticColor::Background).bold();
    let muted = CellStyle::new(SemanticColor::Muted, SemanticColor::Background);
    let success = CellStyle::new(SemanticColor::Success, SemanticColor::Background);
    let warning = CellStyle::new(SemanticColor::Warning, SemanticColor::Background);
    let danger = CellStyle::new(SemanticColor::Danger, SemanticColor::Background);

    display.clear(background);
    display.border(
        GridRect::new(0, 0, DISPLAY_WIDTH, DISPLAY_HEIGHT),
        BorderStyle::ascii(primary),
    )?;
    display.text(GridPoint::new(3, 2), "RECICA COMPUTER WORKS", primary)?;
    display.text(
        GridPoint::new(3, 4),
        "DRX-90 DISPLAY DIAGNOSTIC / 100 X 36",
        background,
    )?;
    display.text(
        GridPoint::new(3, 6),
        "HOST-INDEPENDENT CELL COMPOSITION",
        muted,
    )?;
    display.text(GridPoint::new(3, 9), "PRIMARY LINK", primary)?;
    display.text(GridPoint::new(3, 11), "SYSTEM READY", success)?;
    display.text(GridPoint::new(3, 13), "SIGNAL WARNING", warning)?;
    display.text(GridPoint::new(3, 15), "LINK FAILURE", danger)?;
    display.text(
        GridPoint::new(3, 19),
        "GLYPHS: ABCDEFGHIJKLMNOPQRSTUVWXYZ 0123456789",
        background,
    )?;
    display.text(
        GridPoint::new(3, 21),
        "MARKS : !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~",
        background,
    )?;
    display.text(
        GridPoint::new(3, 32),
        "ESC / Q / CTRL+C  RETURN TO HOST",
        muted,
    )?;
    display.text(GridPoint::new(74, 32), "R/OS VIDEO TEST", muted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DISPLAY_SIZE, DisplayBuffer};

    #[test]
    fn diagnostic_grid_has_canonical_geometry() {
        let mut display = DisplayBuffer::canonical();
        render_diagnostic_grid(&mut display).expect("diagnostic glyphs are valid");
        let snapshot = display.snapshot();

        assert_eq!(snapshot.size, DISPLAY_SIZE);
        assert_eq!(snapshot.cells.len(), DISPLAY_SIZE.area());
        assert_eq!(
            snapshot.character_grid().lines().count(),
            DISPLAY_HEIGHT as usize
        );
        assert!(
            snapshot
                .character_grid()
                .contains("DRX-90 DISPLAY DIAGNOSTIC / 100 X 36")
        );
    }
}
