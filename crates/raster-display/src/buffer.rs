// SPDX-License-Identifier: MPL-2.0

use crate::{
    BorderStyle, CellStyle, GameCell, GlyphError, GridPoint, GridRect, GridSize, TextStyle,
};

/// Drawing interface consumed by system screens and games.
pub trait Display {
    fn size(&self) -> GridSize;
    fn clear(&mut self, style: CellStyle);
    fn put(&mut self, point: GridPoint, cell: GameCell) -> bool;
    fn text(&mut self, point: GridPoint, text: &str, style: TextStyle) -> Result<(), GlyphError>;
    fn fill_rect(&mut self, rect: GridRect, cell: GameCell);
    fn border(&mut self, rect: GridRect, style: BorderStyle) -> Result<(), GlyphError>;
}

/// An owned, host-independent rectangular cell buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplayBuffer {
    size: GridSize,
    cells: Vec<GameCell>,
}

impl DisplayBuffer {
    #[must_use]
    pub fn new(size: GridSize) -> Self {
        Self {
            size,
            cells: vec![GameCell::default(); size.area()],
        }
    }

    #[must_use]
    pub fn canonical() -> Self {
        Self::new(crate::DISPLAY_SIZE)
    }

    #[must_use]
    pub fn get(&self, point: GridPoint) -> Option<&GameCell> {
        self.index(point).map(|index| &self.cells[index])
    }

    #[must_use]
    pub fn cells(&self) -> &[GameCell] {
        &self.cells
    }

    #[must_use]
    pub fn snapshot(&self) -> DisplaySnapshot {
        DisplaySnapshot {
            size: self.size,
            cells: self.cells.clone(),
        }
    }

    #[must_use]
    pub fn viewport(&mut self, rect: GridRect) -> DisplayViewport<'_> {
        let clip = rect.intersection(GridRect::from_size(self.size));
        DisplayViewport {
            buffer: self,
            origin: rect.origin,
            clip,
        }
    }

    fn index(&self, point: GridPoint) -> Option<usize> {
        self.size
            .contains(point)
            .then_some(point.y as usize * self.size.width as usize + point.x as usize)
    }

    fn put_clipped(&mut self, point: GridPoint, cell: GameCell, clip: GridRect) -> bool {
        if !clip.contains(point) {
            return false;
        }
        let Some(index) = self.index(point) else {
            return false;
        };
        self.cells[index] = cell;
        true
    }
}

impl Display for DisplayBuffer {
    fn size(&self) -> GridSize {
        self.size
    }

    fn clear(&mut self, style: CellStyle) {
        self.cells.fill(GameCell::space(style));
    }

    fn put(&mut self, point: GridPoint, cell: GameCell) -> bool {
        self.put_clipped(point, cell, GridRect::from_size(self.size))
    }

    fn text(&mut self, point: GridPoint, text: &str, style: TextStyle) -> Result<(), GlyphError> {
        draw_text(self, GridRect::from_size(self.size), point, text, style)
    }

    fn fill_rect(&mut self, rect: GridRect, cell: GameCell) {
        fill_rect(self, GridRect::from_size(self.size), rect, cell);
    }

    fn border(&mut self, rect: GridRect, style: BorderStyle) -> Result<(), GlyphError> {
        draw_border(self, GridRect::from_size(self.size), rect, style)
    }
}

/// A clipped local-coordinate view into a display buffer.
pub struct DisplayViewport<'a> {
    buffer: &'a mut DisplayBuffer,
    origin: GridPoint,
    clip: GridRect,
}

impl DisplayViewport<'_> {
    fn translate(&self, point: GridPoint) -> GridPoint {
        GridPoint::new(
            self.origin.x.saturating_add(point.x),
            self.origin.y.saturating_add(point.y),
        )
    }
}

impl Display for DisplayViewport<'_> {
    fn size(&self) -> GridSize {
        self.clip.size
    }

    fn clear(&mut self, style: CellStyle) {
        fill_rect(self.buffer, self.clip, self.clip, GameCell::space(style));
    }

    fn put(&mut self, point: GridPoint, cell: GameCell) -> bool {
        self.buffer
            .put_clipped(self.translate(point), cell, self.clip)
    }

    fn text(&mut self, point: GridPoint, text: &str, style: TextStyle) -> Result<(), GlyphError> {
        draw_text(self.buffer, self.clip, self.translate(point), text, style)
    }

    fn fill_rect(&mut self, rect: GridRect, cell: GameCell) {
        let translated = GridRect {
            origin: self.translate(rect.origin),
            size: rect.size,
        };
        fill_rect(self.buffer, self.clip, translated, cell);
    }

    fn border(&mut self, rect: GridRect, style: BorderStyle) -> Result<(), GlyphError> {
        let translated = GridRect {
            origin: self.translate(rect.origin),
            size: rect.size,
        };
        draw_border(self.buffer, self.clip, translated, style)
    }
}

/// Immutable structured capture of a display buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisplaySnapshot {
    pub size: GridSize,
    pub cells: Vec<GameCell>,
}

impl DisplaySnapshot {
    #[must_use]
    pub fn character_grid(&self) -> String {
        let row_width = self.size.width as usize;
        let mut output =
            String::with_capacity(self.cells.len() + self.size.height.saturating_sub(1) as usize);
        for (index, row) in self.cells.chunks(row_width).enumerate() {
            if index > 0 {
                output.push('\n');
            }
            output.extend(row.iter().map(|cell| cell.glyph()));
        }
        output
    }
}

fn draw_text(
    buffer: &mut DisplayBuffer,
    clip: GridRect,
    point: GridPoint,
    text: &str,
    style: TextStyle,
) -> Result<(), GlyphError> {
    let cells = text
        .chars()
        .map(|glyph| GameCell::new(glyph, style))
        .collect::<Result<Vec<_>, _>>()?;

    for (offset, cell) in cells.into_iter().enumerate() {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        let target = GridPoint::new(point.x.saturating_add(offset), point.y);
        if target.x == u16::MAX && offset > 0 {
            break;
        }
        buffer.put_clipped(target, cell, clip);
    }
    Ok(())
}

fn fill_rect(buffer: &mut DisplayBuffer, clip: GridRect, rect: GridRect, cell: GameCell) {
    let area = rect.intersection(clip);
    for y in area.origin.y..area.bottom() {
        for x in area.origin.x..area.right() {
            buffer.put_clipped(GridPoint::new(x, y), cell, clip);
        }
    }
}

fn draw_border(
    buffer: &mut DisplayBuffer,
    clip: GridRect,
    rect: GridRect,
    style: BorderStyle,
) -> Result<(), GlyphError> {
    style.glyphs.validate()?;
    if rect.size.width < 2 || rect.size.height < 2 {
        return Ok(());
    }

    let left = rect.origin.x;
    let top = rect.origin.y;
    let right = rect.right().saturating_sub(1);
    let bottom = rect.bottom().saturating_sub(1);
    let cell = |glyph| GameCell::new(glyph, style.cell_style);

    for x in left.saturating_add(1)..right {
        buffer.put_clipped(GridPoint::new(x, top), cell(style.glyphs.horizontal)?, clip);
        buffer.put_clipped(
            GridPoint::new(x, bottom),
            cell(style.glyphs.horizontal)?,
            clip,
        );
    }
    for y in top.saturating_add(1)..bottom {
        buffer.put_clipped(GridPoint::new(left, y), cell(style.glyphs.vertical)?, clip);
        buffer.put_clipped(GridPoint::new(right, y), cell(style.glyphs.vertical)?, clip);
    }

    buffer.put_clipped(
        GridPoint::new(left, top),
        cell(style.glyphs.top_left)?,
        clip,
    );
    buffer.put_clipped(
        GridPoint::new(right, top),
        cell(style.glyphs.top_right)?,
        clip,
    );
    buffer.put_clipped(
        GridPoint::new(left, bottom),
        cell(style.glyphs.bottom_left)?,
        clip,
    );
    buffer.put_clipped(
        GridPoint::new(right, bottom),
        cell(style.glyphs.bottom_right)?,
        clip,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SemanticColor;

    fn cell(glyph: char) -> GameCell {
        GameCell::new(glyph, CellStyle::default()).expect("test glyph is valid")
    }

    #[test]
    fn out_of_bounds_put_is_ignored() {
        let mut buffer = DisplayBuffer::new(GridSize::new(2, 2));

        assert!(!buffer.put(GridPoint::new(2, 0), cell('X')));
        assert_eq!(buffer.snapshot().character_grid(), "  \n  ");
    }

    #[test]
    fn invalid_text_does_not_partially_modify_buffer() {
        let mut buffer = DisplayBuffer::new(GridSize::new(4, 1));

        assert!(
            buffer
                .text(GridPoint::new(0, 0), "A界", CellStyle::default())
                .is_err()
        );
        assert_eq!(buffer.snapshot().character_grid(), "    ");
    }

    #[test]
    fn viewport_uses_local_coordinates_and_clips() {
        let mut buffer = DisplayBuffer::new(GridSize::new(5, 3));
        {
            let mut viewport = buffer.viewport(GridRect::new(2, 1, 2, 2));
            viewport
                .text(GridPoint::new(0, 0), "ABC", CellStyle::default())
                .expect("ASCII is valid");
        }

        assert_eq!(buffer.snapshot().character_grid(), "     \n  AB \n     ");
    }

    #[test]
    fn border_draws_expected_ascii_cells() {
        let mut buffer = DisplayBuffer::new(GridSize::new(5, 3));
        buffer
            .border(
                GridRect::new(0, 0, 5, 3),
                BorderStyle::ascii(CellStyle::new(
                    SemanticColor::Primary,
                    SemanticColor::Background,
                )),
            )
            .expect("ASCII border is valid");

        assert_eq!(buffer.snapshot().character_grid(), "+---+\n|   |\n+---+");
    }
}
