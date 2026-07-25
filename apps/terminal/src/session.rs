// SPDX-License-Identifier: MPL-2.0

use std::{
    io::{self, Write},
    panic,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    event::{
        DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, queue,
    style::ResetColor,
    terminal::{
        DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement,
    },
};
use raster_engine::InputCapability;

static ACTIVE_SESSION: AtomicBool = AtomicBool::new(false);
static ENHANCED_KEYBOARD: AtomicBool = AtomicBool::new(false);

pub struct TerminalSession {
    raw_mode: bool,
    alternate_screen: bool,
    keyboard_enhanced: bool,
    mouse_capture: bool,
    cursor_hidden: bool,
    line_wrap_disabled: bool,
}

impl TerminalSession {
    pub fn enter() -> Result<(Self, InputCapability)> {
        let mut session = Self {
            raw_mode: false,
            alternate_screen: false,
            keyboard_enhanced: false,
            mouse_capture: false,
            cursor_hidden: false,
            line_wrap_disabled: false,
        };

        enable_raw_mode().context("failed to enable terminal raw mode")?;
        session.raw_mode = true;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
        session.alternate_screen = true;

        execute!(stdout, DisableLineWrap).context("failed to disable terminal line wrapping")?;
        session.line_wrap_disabled = true;

        let keyboard_enhanced = supports_keyboard_enhancement().unwrap_or(false);
        if keyboard_enhanced {
            execute!(
                stdout,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )
            .context("failed to enable enhanced keyboard reporting")?;
            session.keyboard_enhanced = true;
        }

        execute!(stdout, Hide).context("failed to hide terminal cursor")?;
        session.cursor_hidden = true;

        execute!(stdout, EnableMouseCapture).context("failed to enable mouse capture")?;
        session.mouse_capture = true;
        stdout.flush().context("failed to flush terminal setup")?;

        ENHANCED_KEYBOARD.store(session.keyboard_enhanced, Ordering::SeqCst);
        ACTIVE_SESSION.store(true, Ordering::SeqCst);
        let capability = if session.keyboard_enhanced {
            InputCapability::Enhanced
        } else {
            InputCapability::Compatibility
        };
        Ok((session, capability))
    }

    pub fn install_panic_cleanup(&self) {
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(move |information| {
            restore_active_terminal();
            previous_hook(information);
        }));
    }

    fn restore_partial(&mut self) {
        let mut stdout = io::stdout();
        if self.keyboard_enhanced {
            let _ = queue!(stdout, PopKeyboardEnhancementFlags);
            self.keyboard_enhanced = false;
        }
        if self.mouse_capture {
            let _ = queue!(stdout, DisableMouseCapture);
            self.mouse_capture = false;
        }
        if self.cursor_hidden {
            let _ = queue!(stdout, Show);
            self.cursor_hidden = false;
        }
        if self.line_wrap_disabled {
            let _ = queue!(stdout, EnableLineWrap);
            self.line_wrap_disabled = false;
        }
        if self.alternate_screen {
            let _ = queue!(stdout, LeaveAlternateScreen);
            self.alternate_screen = false;
        }
        let _ = queue!(stdout, ResetColor);
        let _ = stdout.flush();
        if self.raw_mode {
            let _ = disable_raw_mode();
            self.raw_mode = false;
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if ACTIVE_SESSION.swap(false, Ordering::SeqCst) {
            restore_terminal(self.keyboard_enhanced);
            self.raw_mode = false;
            self.alternate_screen = false;
            self.keyboard_enhanced = false;
            self.mouse_capture = false;
            self.cursor_hidden = false;
            self.line_wrap_disabled = false;
        } else {
            self.restore_partial();
        }
    }
}

fn restore_active_terminal() {
    if ACTIVE_SESSION.swap(false, Ordering::SeqCst) {
        restore_terminal(ENHANCED_KEYBOARD.swap(false, Ordering::SeqCst));
    }
}

fn restore_terminal(keyboard_enhanced: bool) {
    let mut stdout = io::stdout();
    if keyboard_enhanced {
        let _ = queue!(stdout, PopKeyboardEnhancementFlags);
    }
    let _ = execute!(
        stdout,
        DisableMouseCapture,
        Show,
        EnableLineWrap,
        ResetColor,
        LeaveAlternateScreen
    );
    let _ = disable_raw_mode();
}
