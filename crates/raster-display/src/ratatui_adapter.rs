// SPDX-License-Identifier: MPL-2.0

use ratatui::{
    buffer::Buffer,
    layout::Position,
    style::{Color, Modifier, Style},
};

use crate::{CellModifiers, Display, DisplayBuffer, GridPoint, SemanticColor};

/// Copies the project cell buffer into a Ratatui target at the requested origin.
pub fn copy_to_ratatui(source: &DisplayBuffer, target: &mut Buffer, origin: GridPoint) {
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
                .set_style(to_ratatui_style(source_cell.style));
        }
    }
}

fn to_ratatui_style(style: crate::CellStyle) -> Style {
    Style::default()
        .fg(to_ratatui_color(style.foreground))
        .bg(to_ratatui_color(style.background))
        .add_modifier(to_ratatui_modifiers(style.modifiers))
}

const fn to_ratatui_color(color: SemanticColor) -> Color {
    match color {
        SemanticColor::Background => Color::Rgb(2, 8, 10),
        SemanticColor::Surface => Color::Rgb(10, 20, 24),
        SemanticColor::Text => Color::Rgb(205, 219, 214),
        SemanticColor::Muted => Color::Rgb(103, 126, 124),
        SemanticColor::Primary => Color::Rgb(69, 224, 211),
        SemanticColor::Secondary => Color::Rgb(76, 139, 245),
        SemanticColor::Accent => Color::Rgb(214, 100, 255),
        SemanticColor::Success => Color::Rgb(98, 214, 130),
        SemanticColor::Warning => Color::Rgb(242, 193, 76),
        SemanticColor::Danger => Color::Rgb(238, 92, 92),
    }
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
    use crate::{CellStyle, Display, GameCell, GridSize};

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

        copy_to_ratatui(&source, &mut target, GridPoint::new(0, 0));

        assert_eq!(target[(0, 0)].symbol(), "X");
        assert_eq!(target[(0, 0)].fg, Color::Rgb(242, 193, 76));
        assert!(target[(0, 0)].modifier.contains(Modifier::BOLD));
    }
}
