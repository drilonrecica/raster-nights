// SPDX-License-Identifier: MPL-2.0

use raster_display::{
    BorderStyle, CellStyle, Display, GameCell, GlyphError, GridPoint, GridRect, SemanticColor,
};

use super::{
    ARENA_HEIGHT, ARENA_WIDTH, Loopback, LoopbackStatus, PORTS, Point, RUN_DURATION_TICKS,
};

const BOARD_X: u16 = 25;
const BOARD_Y: u16 = 4;

pub fn render(game: &Loopback, display: &mut dyn Display) -> Result<(), GlyphError> {
    display.clear(style(SemanticColor::Text));
    display.border(
        GridRect::new(0, 0, 100, 36),
        BorderStyle::ascii(style(SemanticColor::Primary)),
    )?;
    display.text(
        GridPoint::new(3, 0),
        " LOOPBACK / QUICK CIRCUIT ",
        style(SemanticColor::Primary).bold(),
    )?;

    panel(display, GridRect::new(3, 4, 18, 12), "CIRCUIT")?;
    value(display, 5, 7, "SCORE", &format!("{:08}", game.score()))?;
    value(
        display,
        5,
        10,
        "PAYLOADS",
        &format!("{:03}", game.payloads_collected()),
    )?;
    value(display, 5, 13, "NEXT", &format!("x{}", game.multiplier()))?;

    display.border(
        GridRect::new(BOARD_X, BOARD_Y, 50, 22),
        BorderStyle::ascii(style(SemanticColor::Muted)),
    )?;
    for (index, pair) in PORTS.iter().enumerate() {
        let glyph = if index == 0 { 'A' } else { 'B' };
        draw_tile(display, pair.first, glyph, SemanticColor::Accent);
        draw_tile(display, pair.second, glyph, SemanticColor::Accent);
    }
    draw_tile(display, game.payload(), '$', SemanticColor::Secondary);
    for (index, point) in game.route().iter().rev().enumerate() {
        let is_head = index + 1 == game.route().len();
        draw_tile(
            display,
            *point,
            if is_head { '@' } else { 'o' },
            if is_head {
                SemanticColor::Primary
            } else {
                SemanticColor::Success
            },
        );
    }

    panel(display, GridRect::new(79, 4, 17, 18), "LINK")?;
    value(display, 81, 7, "TIME", &format_time(game.remaining_ticks()))?;
    value(
        display,
        81,
        10,
        "INTEGRITY",
        &format!("{}/3", game.integrity()),
    )?;
    display.text(
        GridPoint::new(81, 14),
        status_text(game),
        style(status_color(game)).bold(),
    )?;
    if game.recovery_ticks() > 0 {
        display.text(
            GridPoint::new(81, 17),
            &format!("GUARD {:02}", game.recovery_ticks()),
            style(SemanticColor::Warning).bold(),
        )?;
    }

    display.text(
        GridPoint::new(4, 29),
        "PORTS A/A + B/B PRESERVE HEADING AND BOOST THE NEXT PAYLOAD",
        style(SemanticColor::Muted),
    )?;
    display.text(
        GridPoint::new(4, 32),
        "ARROWS / HJKL  Route heading                         ESC  Pause",
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
        GridPoint::new(x, y + 1),
        value,
        style(SemanticColor::Text).bold(),
    )
}

fn draw_tile(display: &mut dyn Display, point: Point, glyph: char, color: SemanticColor) {
    if !(0..ARENA_WIDTH).contains(&point.x) || !(0..ARENA_HEIGHT).contains(&point.y) {
        return;
    }
    let cell = GameCell::new(glyph, style(color).bold()).expect("ASCII game glyph is canonical");
    let x = BOARD_X + 1 + point.x as u16 * 2;
    let y = BOARD_Y + 1 + point.y as u16;
    let _ = display.put(GridPoint::new(x, y), cell);
    let _ = display.put(GridPoint::new(x + 1, y), cell);
}

fn status_text(game: &Loopback) -> &'static str {
    match game.status() {
        LoopbackStatus::Running if game.recovery_ticks() > 0 => "RECOVERY",
        LoopbackStatus::Running => "LINK STABLE",
        LoopbackStatus::Paused => "PAUSED",
        LoopbackStatus::Completed => "COMPLETE",
        LoopbackStatus::Disconnected => "DISCONNECTED",
    }
}

fn status_color(game: &Loopback) -> SemanticColor {
    match game.status() {
        LoopbackStatus::Running if game.recovery_ticks() > 0 => SemanticColor::Warning,
        LoopbackStatus::Running => SemanticColor::Success,
        LoopbackStatus::Paused => SemanticColor::Warning,
        LoopbackStatus::Completed => SemanticColor::Primary,
        LoopbackStatus::Disconnected => SemanticColor::Danger,
    }
}

fn format_time(remaining_ticks: u64) -> String {
    let seconds = remaining_ticks.min(RUN_DURATION_TICKS).div_ceil(60);
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

const fn style(foreground: SemanticColor) -> CellStyle {
    CellStyle::new(foreground, SemanticColor::Background)
}

#[cfg(test)]
mod tests {
    use super::*;
    use raster_display::{DisplayBuffer, DisplaySnapshot};
    use raster_engine::{GameAction, RunSeed, SimulationStep, SimulationTick};

    #[test]
    fn initial_snapshot_has_double_width_arena_and_accessible_symbols() {
        let game = Loopback::new(RunSeed(11));
        let snapshot = snapshot(&game);
        let grid = snapshot.character_grid();

        assert_eq!(grid.lines().count(), 36);
        assert!(grid.contains("LOOPBACK / QUICK CIRCUIT"));
        assert!(grid.contains("02:00"));
        assert!(grid.contains("AA"));
        assert!(grid.contains("BB"));
        assert!(grid.contains("@@"));
        assert!(grid.contains("$$"));
        assert_eq!(
            snapshot_hash(&snapshot),
            14_906_253_628_957_124_186,
            "\n{grid}"
        );
    }

    #[test]
    fn active_snapshot_changes_route_and_time() {
        let mut game = Loopback::new(RunSeed(12));
        for tick in 1..=120 {
            if tick == 48 {
                game.handle_action(GameAction::MoveDown);
            }
            game.update(SimulationStep {
                tick: SimulationTick(tick),
            });
        }
        let snapshot = snapshot(&game);

        assert!(snapshot.character_grid().contains("01:58"));
        assert_eq!(
            snapshot_hash(&snapshot),
            225_953_944_208_726_836,
            "\n{}",
            snapshot.character_grid()
        );
    }

    #[test]
    fn recovery_snapshot_uses_visible_text_not_color_alone() {
        let mut game = Loopback::new(RunSeed(13));
        game.recovery_ticks = 42;
        game.integrity = 2;
        let snapshot = snapshot(&game);
        let grid = snapshot.character_grid();

        assert!(grid.contains("RECOVERY"));
        assert!(grid.contains("GUARD 42"));
        assert!(grid.contains("2/3"));
        assert_eq!(
            snapshot_hash(&snapshot),
            5_574_283_344_875_418_300,
            "\n{grid}"
        );
    }

    #[test]
    fn paused_snapshot_has_explicit_state() {
        let mut game = Loopback::new(RunSeed(14));
        game.set_paused(true);
        let snapshot = snapshot(&game);

        assert!(snapshot.character_grid().contains("PAUSED"));
        assert_eq!(
            snapshot_hash(&snapshot),
            13_286_054_725_332_061_085,
            "\n{}",
            snapshot.character_grid()
        );
    }

    #[test]
    fn game_over_snapshot_has_explicit_state() {
        let mut game = Loopback::new(RunSeed(15));
        game.status = LoopbackStatus::Disconnected;
        game.integrity = 0;
        let snapshot = snapshot(&game);

        assert!(snapshot.character_grid().contains("DISCONNECTED"));
        assert_eq!(
            snapshot_hash(&snapshot),
            16_998_600_491_811_311_657,
            "\n{}",
            snapshot.character_grid()
        );
    }

    fn snapshot(game: &Loopback) -> DisplaySnapshot {
        let mut display = DisplayBuffer::canonical();
        render(game, &mut display).expect("Loopback render should be valid");
        display.snapshot()
    }

    fn snapshot_hash(snapshot: &DisplaySnapshot) -> u64 {
        let mut hash = 14_695_981_039_346_656_037_u64;
        for value in [snapshot.size.width, snapshot.size.height] {
            for byte in value.to_le_bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211);
            }
        }
        for cell in &snapshot.cells {
            for byte in u32::from(cell.glyph()).to_le_bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211);
            }
            for byte in [
                cell.style.foreground as u8,
                cell.style.background as u8,
                u8::from(cell.style.modifiers.bold)
                    | u8::from(cell.style.modifiers.dim) << 1
                    | u8::from(cell.style.modifiers.underlined) << 2
                    | u8::from(cell.style.modifiers.reversed) << 3,
            ] {
                hash = (hash ^ u64::from(byte)).wrapping_mul(1_099_511_628_211);
            }
        }
        hash
    }
}
