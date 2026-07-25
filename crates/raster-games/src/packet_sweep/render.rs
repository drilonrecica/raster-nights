// SPDX-License-Identifier: MPL-2.0

use raster_display::{
    BorderStyle, CellStyle, Display, GlyphError, GridPoint, GridRect, SemanticColor,
};

use super::{ARENA_HEIGHT, ARENA_WIDTH, PacketSweep, PacketSweepStatus, Point, is_wall};

const ORIGIN_X: u16 = 25;
const ORIGIN_Y: u16 = 7;

pub fn render(game: &PacketSweep, display: &mut dyn Display) -> Result<(), GlyphError> {
    display.clear(CellStyle::new(
        SemanticColor::Text,
        SemanticColor::Background,
    ));
    display.border(
        GridRect::new(0, 0, 100, 36),
        BorderStyle::ascii(CellStyle::new(
            SemanticColor::Primary,
            SemanticColor::Background,
        )),
    )?;
    display.text(
        GridPoint::new(3, 2),
        "PACKET SWEEP / MAINTENANCE RUN",
        CellStyle::new(SemanticColor::Primary, SemanticColor::Background).bold(),
    )?;
    display.text(
        GridPoint::new(69, 2),
        &format!(
            "TIME {:02}:{:02}",
            game.remaining_ticks() / 3_600,
            (game.remaining_ticks() / 60) % 60
        ),
        CellStyle::new(SemanticColor::Text, SemanticColor::Background),
    )?;
    display.text(
        GridPoint::new(3, 4),
        &format!(
            "SCORE {:08}  PACKETS {:03}  STREAK {:02}  INTEGRITY {}",
            game.score(),
            game.collected(),
            game.streak(),
            game.integrity()
        ),
        CellStyle::new(
            if game.integrity() == 1 {
                SemanticColor::Danger
            } else {
                SemanticColor::Text
            },
            SemanticColor::Background,
        ),
    )?;

    for y in 0..ARENA_HEIGHT {
        for x in 0..ARENA_WIDTH {
            let point = Point::new(x, y);
            if is_wall(point) {
                tile(display, point, "##", SemanticColor::Muted)?;
            }
        }
    }
    tile(display, game.packet(), "<>", SemanticColor::Success)?;
    for error in game.errors() {
        tile(display, error.position, "!!", SemanticColor::Danger)?;
    }
    let cursor_color = if game.recovery_ticks() > 0 {
        SemanticColor::Warning
    } else {
        SemanticColor::Primary
    };
    tile(display, game.cursor(), "[]", cursor_color)?;

    let status = match game.status() {
        PacketSweepStatus::Running if game.recovery_ticks() > 0 => {
            format!(
                "RECOVERY {:02} / CHECKSUM COLLISION PROTECTION",
                game.recovery_ticks()
            )
        }
        PacketSweepStatus::Running => "ACTIVE / COLLECT <> / AVOID !!".to_owned(),
        PacketSweepStatus::Paused => "PAUSED / MAINTENANCE CLOCK SUSPENDED".to_owned(),
        PacketSweepStatus::Completed => "RUN COMPLETE / MAINTENANCE WINDOW CLOSED".to_owned(),
        PacketSweepStatus::Failed => "GAME OVER / CURSOR INTEGRITY LOST".to_owned(),
    };
    display.text(
        GridPoint::new(3, 33),
        &status,
        CellStyle::new(
            if matches!(game.status(), PacketSweepStatus::Failed) {
                SemanticColor::Danger
            } else {
                SemanticColor::Warning
            },
            SemanticColor::Background,
        )
        .bold(),
    )?;
    Ok(())
}

fn tile(
    display: &mut dyn Display,
    point: Point,
    glyphs: &str,
    color: SemanticColor,
) -> Result<(), GlyphError> {
    display.text(
        GridPoint::new(
            ORIGIN_X + u16::try_from(point.x).expect("arena x is nonnegative") * 2,
            ORIGIN_Y + u16::try_from(point.y).expect("arena y is nonnegative"),
        ),
        glyphs,
        CellStyle::new(color, SemanticColor::Background).bold(),
    )
}
