// SPDX-License-Identifier: MPL-2.0

use raster_display::{
    BorderStyle, CellStyle, Display, GameCell, GlyphError, GridPoint, GridRect, SemanticColor,
};

use super::{HIDDEN_ROWS, MATRIX_WIDTH, Packet, SignalStack, VISIBLE_ROWS};

const BOARD_X: u16 = 38;
const BOARD_Y: u16 = 4;

pub fn render(game: &SignalStack, display: &mut dyn Display) -> Result<(), GlyphError> {
    display.clear(style(SemanticColor::Text));
    display.border(
        GridRect::new(0, 0, 100, 36),
        BorderStyle::ascii(style(SemanticColor::Primary)),
    )?;
    display.text(
        GridPoint::new(3, 0),
        " SIGNAL STACK / STANDARD TRANSMISSION ",
        style(SemanticColor::Primary).bold(),
    )?;

    panel(display, GridRect::new(3, 4, 29, 13), "TRANSMISSION")?;
    value(display, 6, 7, "SCORE", &format!("{:010}", game.score()))?;
    value(display, 6, 10, "RATE", &format!("{:02}", game.rate()))?;
    value(
        display,
        6,
        13,
        "CHANNELS",
        &format!("{:04}", game.cleared_channels()),
    )?;

    panel(display, GridRect::new(3, 20, 29, 11), "HOLD BUFFER")?;
    let held = game.hold().map_or("--".to_owned(), packet_text);
    display.text(
        GridPoint::new(14, 25),
        &held,
        style(if game.hold_available() {
            SemanticColor::Secondary
        } else {
            SemanticColor::Muted
        })
        .bold(),
    )?;

    display.border(
        GridRect::new(BOARD_X, BOARD_Y, 22, 22),
        BorderStyle::ascii(style(SemanticColor::Primary)),
    )?;
    for y in 0..VISIBLE_ROWS {
        for x in 0..MATRIX_WIDTH {
            if let Some(packet) = game.cell(x, y + HIDDEN_ROWS) {
                draw_packet(display, x, y, packet);
            }
        }
    }
    if let Some(active) = game.active() {
        for cell in active.cells() {
            if cell.y >= HIDDEN_ROWS {
                draw_packet(display, cell.x, cell.y - HIDDEN_ROWS, active.packet);
            }
        }
    }

    panel(display, GridRect::new(66, 4, 30, 22), "NEXT PACKETS")?;
    for (index, packet) in game.previews().iter().enumerate() {
        display.text(
            GridPoint::new(73, 8 + index as u16 * 3),
            &format!("{:02}   {}", index + 1, packet_text(*packet)),
            style(packet_color(*packet)).bold(),
        )?;
    }

    display.text(
        GridPoint::new(4, 33),
        "LEFT/RIGHT Move   UP/X,Z Rotate   DOWN Soft drop   SPACE Hard drop   C Hold   ESC Pause",
        style(SemanticColor::Muted),
    )?;
    Ok(())
}

fn panel(display: &mut dyn Display, rect: GridRect, title: &str) -> Result<(), GlyphError> {
    display.border(rect, BorderStyle::ascii(style(SemanticColor::Muted)))?;
    display.text(
        GridPoint::new(rect.origin.x + 2, rect.origin.y),
        &format!(" {title} "),
        style(SemanticColor::Primary).bold(),
    )
}

fn value(
    display: &mut dyn Display,
    x: u16,
    y: u16,
    label: &str,
    value: &str,
) -> Result<(), GlyphError> {
    display.text(GridPoint::new(x, y), label, style(SemanticColor::Muted))?;
    display.text(
        GridPoint::new(x + 11, y),
        value,
        style(SemanticColor::Text).bold(),
    )
}

fn draw_packet(display: &mut dyn Display, x: i8, y: i8, packet: Packet) {
    if !(0..MATRIX_WIDTH).contains(&x) || !(0..VISIBLE_ROWS).contains(&y) {
        return;
    }
    let style = style(packet_color(packet)).bold();
    let glyph = packet_glyph(packet);
    let screen_x = BOARD_X + 1 + x as u16 * 2;
    let screen_y = BOARD_Y + 1 + y as u16;
    let cell = GameCell::new(glyph, style).expect("ASCII packet glyph is canonical");
    let _ = display.put(GridPoint::new(screen_x, screen_y), cell);
    let _ = display.put(GridPoint::new(screen_x + 1, screen_y), cell);
}

const fn packet_glyph(packet: Packet) -> char {
    match packet {
        Packet::I => 'I',
        Packet::J => 'J',
        Packet::L => 'L',
        Packet::O => 'O',
        Packet::S => 'S',
        Packet::T => 'T',
        Packet::Z => 'Z',
    }
}

fn packet_text(packet: Packet) -> String {
    let glyph = packet_glyph(packet);
    format!("{glyph}{glyph}")
}

const fn packet_color(packet: Packet) -> SemanticColor {
    match packet {
        Packet::I => SemanticColor::Secondary,
        Packet::J => SemanticColor::Primary,
        Packet::L => SemanticColor::Warning,
        Packet::O => SemanticColor::Accent,
        Packet::S => SemanticColor::Success,
        Packet::T => SemanticColor::Danger,
        Packet::Z => SemanticColor::Text,
    }
}

const fn style(foreground: SemanticColor) -> CellStyle {
    CellStyle::new(foreground, SemanticColor::Background)
}

#[cfg(test)]
mod tests {
    use super::*;
    use raster_display::DisplayBuffer;
    use raster_engine::RunSeed;

    #[test]
    fn empty_run_snapshot_has_board_hud_and_symbol_redundancy() {
        let game = SignalStack::new(RunSeed(11));
        let mut display = DisplayBuffer::canonical();
        render(&game, &mut display).expect("render");
        let grid = display.snapshot().character_grid();

        assert_eq!(grid.lines().count(), 36);
        assert!(grid.contains("SIGNAL STACK / STANDARD TRANSMISSION"));
        assert!(grid.contains("SCORE      0000000000"));
        assert!(
            ["II", "JJ", "LL", "OO", "SS", "TT", "ZZ"]
                .iter()
                .any(|symbol| grid.contains(symbol))
        );
    }
}
