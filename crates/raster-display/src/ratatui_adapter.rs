// SPDX-License-Identifier: MPL-2.0

use ratatui::{
    buffer::Buffer,
    layout::Position,
    style::{Color, Modifier, Style},
};

use crate::{CellModifiers, Display, DisplayBuffer, GridPoint, Palette, RgbColor};

/// Copies the project cell buffer into a Ratatui target at the requested origin.
pub fn copy_to_ratatui(
    source: &DisplayBuffer,
    target: &mut Buffer,
    origin: GridPoint,
    palette: Palette,
) {
    for y in 0..source.size().height {
        for x in 0..source.size().width {
            let source_point = GridPoint::new(x, y);
            let target_position =
                Position::new(origin.x.saturating_add(x), origin.y.saturating_add(y));
            let (Some(source_cell), Some(target_cell)) =
                (source.get(source_point), target.cell_mut(target_position))
            else {
                continue;
            };
            target_cell
                .set_char(source_cell.glyph())
                .set_style(to_ratatui_style(source_cell.style, palette));
        }
    }
}

fn to_ratatui_style(style: crate::CellStyle, palette: Palette) -> Style {
    Style::default()
        .fg(to_ratatui_color(palette.resolve(style.foreground)))
        .bg(to_ratatui_color(palette.resolve(style.background)))
        .add_modifier(to_ratatui_modifiers(style.modifiers))
}

const fn to_ratatui_color(color: RgbColor) -> Color {
    Color::Rgb(color.red, color.green, color.blue)
}

fn to_ratatui_modifiers(modifiers: CellModifiers) -> Modifier {
    let mut result = Modifier::empty();
    result.set(Modifier::BOLD, modifiers.bold);
    result.set(Modifier::DIM, modifiers.dim);
    result.set(Modifier::UNDERLINED, modifiers.underlined);
    result.set(Modifier::REVERSED, modifiers.reversed);
    result
}

#[cfg(test)]
mod tests {
    use ratatui::{buffer::Buffer, layout::Rect};

    use super::*;
    use crate::{CellStyle, Display, GameCell, GridSize, SemanticColor};

    #[test]
    fn adapter_copies_glyph_and_semantic_style() {
        let mut source = DisplayBuffer::new(GridSize::new(1, 1));
        source.put(
            GridPoint::new(0, 0),
            GameCell::new(
                'X',
                CellStyle::new(SemanticColor::Warning, SemanticColor::Surface).bold(),
            )
            .expect("test glyph is valid"),
        );
        let mut target = Buffer::empty(Rect::new(0, 0, 1, 1));

        copy_to_ratatui(
            &source,
            &mut target,
            GridPoint::new(0, 0),
            Palette::rcw_standard(),
        );

        assert_eq!(target[(0, 0)].symbol(), "X");
        assert_eq!(target[(0, 0)].fg, Color::Rgb(242, 193, 76));
        assert!(target[(0, 0)].modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn adapter_applies_selected_palette() {
        let mut source = DisplayBuffer::new(GridSize::new(1, 1));
        source.put(
            GridPoint::new(0, 0),
            GameCell::new(
                'X',
                CellStyle::new(SemanticColor::Text, SemanticColor::Background),
            )
            .expect("test glyph is valid"),
        );
        let mut target = Buffer::empty(Rect::new(0, 0, 1, 1));

        copy_to_ratatui(
            &source,
            &mut target,
            GridPoint::new(0, 0),
            Palette::high_contrast(),
        );

        assert_eq!(target[(0, 0)].fg, Color::Rgb(255, 255, 255));
        assert_eq!(target[(0, 0)].bg, Color::Rgb(0, 0, 0));
    }
}
