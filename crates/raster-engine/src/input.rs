// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// Quality of key transitions reported by a host.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputCapability {
    Enhanced,
    Compatibility,
}

/// Host-neutral physical key code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyCode {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    Character(char),
    Function(u8),
}

/// Modifier state attached to a physical key.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
}

/// Host-neutral key identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct PhysicalKey {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl PhysicalKey {
    #[must_use]
    pub const fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers {
                control: false,
                alt: false,
                shift: false,
            },
        }
    }
}

/// Host-neutral pointer button.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

/// Input transition produced by a platform adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceInput {
    KeyPressed(PhysicalKey),
    KeyRepeated(PhysicalKey),
    KeyReleased(PhysicalKey),
    PointerMoved {
        column: u16,
        row: u16,
    },
    PointerPressed {
        button: PointerButton,
        column: u16,
        row: u16,
    },
    PointerReleased {
        button: PointerButton,
        column: u16,
        row: u16,
    },
    FocusLost,
    FocusGained,
    Resized {
        columns: u16,
        rows: u16,
    },
}

/// Game-level action after input normalization.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    RotateClockwise,
    RotateCounterclockwise,
    SoftDrop,
    HardDrop,
    Hold,
    Primary,
    Secondary,
}

/// Application action consumed by shared state.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppAction {
    NavigateLeft,
    NavigateRight,
    NavigateUp,
    NavigateDown,
    Confirm,
    Back,
    Pause,
    OpenShell,
    OpenSettings,
    Interrupt,
    TextInput(char),
    Game(GameAction),
}
