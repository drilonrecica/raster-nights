// SPDX-License-Identifier: MPL-2.0

mod native_storage;
mod session;

use std::{
    io,
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use crossterm::event::{
    self, Event, KeyCode as CrosstermKeyCode, KeyEvent, KeyEventKind,
    KeyModifiers as CrosstermKeyModifiers, MouseButton, MouseEventKind,
};
use raster_display::{
    CellStyle, DISPLAY_HEIGHT, DISPLAY_WIDTH, Display, DisplayBuffer, GridPoint, SemanticColor,
    copy_to_ratatui, render_diagnostic_grid,
};
use raster_engine::{
    Application, ApplicationRepository, CalendarDate, DeviceInput, FixedStepClock, HostKind,
    InputCapability, InputSystem, KeyCode, KeyModifiers, PhysicalKey, RunMetadataSource, RunSeed,
};
use raster_games::RasterGameRegistry;
use raster_storage::{InMemoryRepository, Repository};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
};

use session::TerminalSession;

use native_storage::NativeByteStorage;

fn main() -> Result<()> {
    let command = parse_arguments()?;

    let (session, capability) = TerminalSession::enter()?;
    session.install_panic_cleanup();

    let result = match command {
        Command::Run => run_application(capability),
        Command::DisplayTest => run_display_test(capability),
    };

    drop(session);
    result
}

fn run_application(capability: InputCapability) -> Result<()> {
    let mut app = native_application();
    let mut input = InputSystem::new(capability);
    let mut clock = FixedStepClock::new();
    let mut display = DisplayBuffer::canonical();
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("failed to create terminal renderer")?;
    let initial = terminal.size().context("failed to read terminal size")?;
    app.handle_resize(initial.width, initial.height);
    let mut previous_frame = Instant::now();

    while !app.exit_requested() {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(previous_frame);
        previous_frame = now;

        while event::poll(Duration::from_millis(0)).context("failed to poll terminal input")? {
            handle_terminal_event(
                event::read().context("failed to read terminal input")?,
                &mut app,
                &mut input,
                clock.current_tick(),
                terminal.size().context("failed to read terminal size")?,
            );
        }

        for step in clock.advance(elapsed, app.is_suspended()).iter() {
            for action in input.advance(step.tick) {
                app.handle_action(action.action, action.phase);
            }
            app.update(step);
        }

        app.render(&mut display)
            .context("failed to compose application display")?;
        terminal
            .autoresize()
            .context("failed to resize terminal renderer")?;
        terminal
            .draw(|frame| render_frame(frame, &display))
            .context("failed to draw application display")?;
        std::thread::sleep(Duration::from_millis(16));
    }

    drop(terminal);
    Ok(())
}

fn native_application() -> Application {
    let (repository, warning): (Box<dyn ApplicationRepository>, Option<String>) =
        match NativeByteStorage::open() {
            Ok(storage) => (Box::new(Repository::new(storage)), None),
            Err(error) => (
                Box::new(InMemoryRepository::default()),
                Some(format!(
                    "Local persistence is unavailable; records are session-only: {error}"
                )),
            ),
        };
    let mut app = Application::with_services(
        HostKind::Native,
        local_date(),
        Box::new(RasterGameRegistry::new()),
        repository,
        Box::new(NativeRunMetadata::new()),
    );
    if let Some(warning) = warning {
        app.report_persistence_unavailable(warning);
    }
    app
}

#[derive(Debug)]
struct NativeRunMetadata {
    state: AtomicU64,
}

impl NativeRunMetadata {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self {
            state: AtomicU64::new(seed),
        }
    }
}

impl RunMetadataSource for NativeRunMetadata {
    fn next_seed(&mut self) -> RunSeed {
        let value = self
            .state
            .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::Relaxed);
        RunSeed(value ^ value.rotate_left(23))
    }

    fn unix_seconds(&self) -> i64 {
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as i64)
    }
}

fn run_display_test(capability: InputCapability) -> Result<()> {
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
    loop {
        terminal
            .draw(|frame| render_frame(frame, &display))
            .context("failed to draw diagnostic display")?;
        if event::poll(Duration::from_millis(100)).context("failed to poll terminal input")?
            && let Event::Key(key) = event::read().context("failed to read terminal input")?
            && diagnostic_should_exit(key)
        {
            break;
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    Run,
    DisplayTest,
}

fn parse_arguments() -> Result<Command> {
    let mut arguments = std::env::args_os();
    let _executable = arguments.next();
    match (arguments.next(), arguments.next()) {
        (None, None) => Ok(Command::Run),
        (Some(command), None) if command == "display-test" => Ok(Command::DisplayTest),
        _ => bail!("usage: raster-nights [display-test]"),
    }
}

fn handle_terminal_event(
    event: Event,
    app: &mut Application,
    input: &mut InputSystem,
    tick: raster_engine::SimulationTick,
    terminal_size: ratatui::layout::Size,
) {
    match event {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Release {
                app.handle_activity();
            }
            let Some(device_input) = map_key_event(key) else {
                return;
            };
            for action in input.handle(device_input, tick, app.input_context()) {
                app.handle_action(action.action, action.phase);
            }
        }
        Event::Mouse(mouse) => {
            if let Some((column, row)) = display_coordinates(
                mouse.column,
                mouse.row,
                terminal_size.width,
                terminal_size.height,
            ) && matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            {
                app.handle_pointer_press(column, row);
            }
        }
        Event::Resize(columns, rows) => app.handle_resize(columns, rows),
        Event::FocusLost => {
            for action in input.release_all(tick) {
                app.handle_action(action.action, action.phase);
            }
        }
        Event::FocusGained | Event::Paste(_) => {}
    }
}

fn map_key_event(key: KeyEvent) -> Option<DeviceInput> {
    let kind = key.kind;
    let physical_key = PhysicalKey {
        code: match key.code {
            CrosstermKeyCode::Left => KeyCode::ArrowLeft,
            CrosstermKeyCode::Right => KeyCode::ArrowRight,
            CrosstermKeyCode::Up => KeyCode::ArrowUp,
            CrosstermKeyCode::Down => KeyCode::ArrowDown,
            CrosstermKeyCode::Enter => KeyCode::Enter,
            CrosstermKeyCode::Esc => KeyCode::Escape,
            CrosstermKeyCode::Char(' ') => KeyCode::Space,
            CrosstermKeyCode::Char(character) => KeyCode::Character(character),
            CrosstermKeyCode::Tab => KeyCode::Tab,
            CrosstermKeyCode::Backspace => KeyCode::Backspace,
            CrosstermKeyCode::Delete => KeyCode::Delete,
            CrosstermKeyCode::Home => KeyCode::Home,
            CrosstermKeyCode::End => KeyCode::End,
            CrosstermKeyCode::F(number) => KeyCode::Function(number),
            _ => return None,
        },
        modifiers: KeyModifiers {
            control: key.modifiers.contains(CrosstermKeyModifiers::CONTROL),
            alt: key.modifiers.contains(CrosstermKeyModifiers::ALT),
            shift: key.modifiers.contains(CrosstermKeyModifiers::SHIFT),
        },
    };
    Some(match kind {
        KeyEventKind::Press => DeviceInput::KeyPressed(physical_key),
        KeyEventKind::Repeat => DeviceInput::KeyRepeated(physical_key),
        KeyEventKind::Release => DeviceInput::KeyReleased(physical_key),
    })
}

fn display_coordinates(
    column: u16,
    row: u16,
    terminal_width: u16,
    terminal_height: u16,
) -> Option<(u16, u16)> {
    if terminal_width < DISPLAY_WIDTH || terminal_height < DISPLAY_HEIGHT {
        return None;
    }
    let origin_x = (terminal_width - DISPLAY_WIDTH) / 2;
    let origin_y = (terminal_height - DISPLAY_HEIGHT) / 2;
    let column = column.checked_sub(origin_x)?;
    let row = row.checked_sub(origin_y)?;
    (column < DISPLAY_WIDTH && row < DISPLAY_HEIGHT).then_some((column, row))
}

fn diagnostic_should_exit(key: KeyEvent) -> bool {
    if key.kind == KeyEventKind::Release {
        return false;
    }
    matches!(
        key.code,
        CrosstermKeyCode::Esc | CrosstermKeyCode::Char('q' | 'Q')
    ) || (key.modifiers.contains(CrosstermKeyModifiers::CONTROL)
        && matches!(key.code, CrosstermKeyCode::Char('c' | 'C')))
}

fn local_date() -> CalendarDate {
    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    CalendarDate::new(now.day(), u8::from(now.month()), now.year())
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
        assert!(!diagnostic_should_exit(KeyEvent::new_with_kind(
            CrosstermKeyCode::Esc,
            CrosstermKeyModifiers::NONE,
            KeyEventKind::Release,
        )));
    }

    #[test]
    fn escape_q_and_control_c_exit() {
        assert!(diagnostic_should_exit(KeyEvent::new(
            CrosstermKeyCode::Esc,
            CrosstermKeyModifiers::NONE
        )));
        assert!(diagnostic_should_exit(KeyEvent::new(
            CrosstermKeyCode::Char('q'),
            CrosstermKeyModifiers::NONE
        )));
        assert!(diagnostic_should_exit(KeyEvent::new(
            CrosstermKeyCode::Char('c'),
            CrosstermKeyModifiers::CONTROL,
        )));
    }

    #[test]
    fn pointer_coordinates_account_for_centered_display() {
        assert_eq!(display_coordinates(10, 5, 120, 46), Some((0, 0)));
        assert_eq!(display_coordinates(109, 40, 120, 46), Some((99, 35)));
        assert_eq!(display_coordinates(9, 5, 120, 46), None);
    }

    #[test]
    fn key_events_are_normalized_before_reaching_engine() {
        let event = KeyEvent::new(CrosstermKeyCode::Char('c'), CrosstermKeyModifiers::CONTROL);
        assert_eq!(
            map_key_event(event),
            Some(DeviceInput::KeyPressed(PhysicalKey {
                code: KeyCode::Character('c'),
                modifiers: KeyModifiers {
                    control: true,
                    alt: false,
                    shift: false,
                },
            }))
        );
    }
}
