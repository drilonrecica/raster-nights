// SPDX-License-Identifier: MPL-2.0

mod session;

use std::{io, time::Duration};

use anyhow::{Context, Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use raster_display::{
    CellStyle, DISPLAY_HEIGHT, DISPLAY_WIDTH, Display, DisplayBuffer, GridPoint, SemanticColor,
    copy_to_ratatui, render_diagnostic_grid,
};
use raster_engine::InputCapability;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
};

use session::TerminalSession;

fn main() -> Result<()> {
    validate_arguments()?;

    let (session, capability) = TerminalSession::enter()?;
    session.install_panic_cleanup();

    let mut display = DisplayBuffer::canonical();
    render_diagnostic_grid(&mut display).context("failed to compose diagnostic display")?;
    display
        .text(
            GridPoint::new(3, 34),
            match capability {
                InputCapability::Enhanced => "INPUT MODE: ENHANCED",
                InputCapability::Compatibility => "INPUT MODE: COMPATIBILITY",
            },
            CellStyle::new(SemanticColor::Muted, SemanticColor::Background),
        )
        .context("failed to compose input diagnostics")?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("failed to create terminal renderer")?;
    terminal
        .draw(|frame| render_frame(frame, &display))
        .context("failed to draw diagnostic display")?;

    loop {
        if !event::poll(Duration::from_millis(100)).context("failed to poll terminal input")? {
            continue;
        }
        match event::read().context("failed to read terminal input")? {
            Event::Key(key) if should_exit(key) => break,
            Event::Resize(_, _) => {
                terminal
                    .autoresize()
                    .context("failed to resize terminal renderer")?;
                terminal
                    .draw(|frame| render_frame(frame, &display))
                    .context("failed to redraw diagnostic display")?;
            }
            _ => {}
        }
    }

    drop(terminal);
    drop(session);
    Ok(())
}

fn validate_arguments() -> Result<()> {
    let mut arguments = std::env::args_os();
    let _executable = arguments.next();
    match (arguments.next(), arguments.next()) {
        (None, None) => Ok(()),
        (Some(command), None) if command == "display-test" => Ok(()),
        _ => bail!("usage: raster-nights [display-test]"),
    }
}

fn should_exit(key: KeyEvent) -> bool {
    if key.kind == KeyEventKind::Release {
        return false;
    }
    matches!(key.code, KeyCode::Esc | KeyCode::Char('q' | 'Q'))
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'C')))
}

fn render_frame(frame: &mut Frame<'_>, source: &DisplayBuffer) {
    let area = frame.area();
    frame.buffer_mut().reset();
    if area.width < DISPLAY_WIDTH || area.height < DISPLAY_HEIGHT {
        let message = format!(
            "DRX-90 REQUIRES {DISPLAY_WIDTH} X {DISPLAY_HEIGHT} / CURRENT {} X {}",
            area.width, area.height
        );
        let x = area.x + area.width.saturating_sub(message.len() as u16) / 2;
        let y = area.y + area.height / 2;
        frame.buffer_mut().set_string(
            x,
            y,
            message,
            Style::default().fg(Color::Yellow).bg(Color::Black),
        );
        return;
    }

    let origin = GridPoint::new(
        area.x + (area.width - DISPLAY_WIDTH) / 2,
        area.y + (area.height - DISPLAY_HEIGHT) / 2,
    );
    copy_to_ratatui(source, frame.buffer_mut(), origin);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_event_does_not_exit() {
        assert!(!should_exit(KeyEvent::new_with_kind(
            KeyCode::Esc,
            KeyModifiers::NONE,
            KeyEventKind::Release,
        )));
    }

    #[test]
    fn escape_q_and_control_c_exit() {
        assert!(should_exit(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(should_exit(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE
        )));
        assert!(should_exit(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )));
    }
}
