// SPDX-License-Identifier: MPL-2.0

use crate::{
    ActionPhase, AppAction, InputContext, LiveRegion, SemanticActionKind, SemanticId, SemanticNode,
    SemanticRole, SemanticState, SemanticUiTree, SimulationStep,
};
use raster_display::{Display, GlyphError};

pub const MINIMUM_COLUMNS: u16 = raster_display::DISPLAY_WIDTH;
pub const MINIMUM_ROWS: u16 = raster_display::DISPLAY_HEIGHT;

const COLD_BOOT_TICKS: u64 = 300;
const WARM_BOOT_TICKS: u64 = 150;
const SHUTDOWN_TICKS: u64 = 90;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostKind {
    Native,
    Browser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalendarDate {
    pub day: u8,
    pub month: u8,
    pub year: i32,
}

impl CalendarDate {
    #[must_use]
    pub const fn new(day: u8, month: u8, year: i32) -> Self {
        Self { day, month, year }
    }

    #[must_use]
    pub const fn is_outside_certified_period(self) -> bool {
        self.year > 1999
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppStateKind {
    PrivacyNotice,
    ColdBoot,
    WarmBoot,
    Launcher,
    SoftwareDetails,
    SystemMenu,
    InterruptConfirm,
    ResizeSuspended,
    Shutdown,
    FatalError,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AppState {
    PrivacyNotice,
    ColdBoot(BootState),
    WarmBoot(BootState),
    Launcher,
    SoftwareDetails,
    SystemMenu(SystemMenuState),
    InterruptConfirm(Box<AppState>),
    ResizeSuspended(ResizeSuspendedState),
    Shutdown(ShutdownState),
    FatalError(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BootState {
    pub(crate) elapsed_ticks: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SystemMenuState {
    pub(crate) selected: SystemMenuItem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SystemMenuItem {
    Return,
    Shutdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResizeSuspendedState {
    pub(crate) previous: Box<AppState>,
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) ready_to_resume: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShutdownState {
    pub(crate) elapsed_ticks: u64,
    pub(crate) urgent: bool,
}

#[derive(Debug)]
pub struct Application {
    state: AppState,
    host: HostKind,
    date: CalendarDate,
    semantic_revision: u64,
    exit_requested: bool,
}

impl Application {
    #[must_use]
    pub fn new(host: HostKind, date: CalendarDate, privacy_acknowledged: bool) -> Self {
        let state = if privacy_acknowledged {
            AppState::ColdBoot(BootState { elapsed_ticks: 0 })
        } else {
            AppState::PrivacyNotice
        };
        Self {
            state,
            host,
            date,
            semantic_revision: 1,
            exit_requested: false,
        }
    }

    #[must_use]
    pub fn warm_boot(host: HostKind, date: CalendarDate) -> Self {
        Self {
            state: AppState::WarmBoot(BootState { elapsed_ticks: 0 }),
            host,
            date,
            semantic_revision: 1,
            exit_requested: false,
        }
    }

    #[must_use]
    pub const fn state_kind(&self) -> AppStateKind {
        match self.state {
            AppState::PrivacyNotice => AppStateKind::PrivacyNotice,
            AppState::ColdBoot(_) => AppStateKind::ColdBoot,
            AppState::WarmBoot(_) => AppStateKind::WarmBoot,
            AppState::Launcher => AppStateKind::Launcher,
            AppState::SoftwareDetails => AppStateKind::SoftwareDetails,
            AppState::SystemMenu(_) => AppStateKind::SystemMenu,
            AppState::InterruptConfirm(_) => AppStateKind::InterruptConfirm,
            AppState::ResizeSuspended(_) => AppStateKind::ResizeSuspended,
            AppState::Shutdown(_) => AppStateKind::Shutdown,
            AppState::FatalError(_) => AppStateKind::FatalError,
        }
    }

    #[must_use]
    pub const fn input_context(&self) -> InputContext {
        InputContext::Navigation
    }

    #[must_use]
    pub const fn is_suspended(&self) -> bool {
        matches!(self.state, AppState::ResizeSuspended(_))
    }

    #[must_use]
    pub const fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    pub fn handle_action(&mut self, action: AppAction, phase: ActionPhase) {
        if phase == ActionPhase::Released {
            return;
        }

        if action == AppAction::Interrupt {
            self.handle_interrupt();
            return;
        }

        match &mut self.state {
            AppState::PrivacyNotice if action == AppAction::Confirm => {
                self.transition(AppState::ColdBoot(BootState { elapsed_ticks: 0 }));
            }
            AppState::ColdBoot(_) | AppState::WarmBoot(_) => {
                self.transition(AppState::Launcher);
            }
            AppState::Launcher => match action {
                AppAction::Confirm => self.transition(AppState::SoftwareDetails),
                AppAction::Back => self.transition(AppState::SystemMenu(SystemMenuState {
                    selected: SystemMenuItem::Return,
                })),
                _ => {}
            },
            AppState::SoftwareDetails if action == AppAction::Back => {
                self.transition(AppState::Launcher);
            }
            AppState::SystemMenu(menu) => match action {
                AppAction::NavigateUp | AppAction::NavigateDown => {
                    menu.selected = match menu.selected {
                        SystemMenuItem::Return => SystemMenuItem::Shutdown,
                        SystemMenuItem::Shutdown => SystemMenuItem::Return,
                    };
                    self.bump_revision();
                }
                AppAction::Confirm => match menu.selected {
                    SystemMenuItem::Return => self.transition(AppState::Launcher),
                    SystemMenuItem::Shutdown => self.begin_shutdown(false),
                },
                AppAction::Back => self.transition(AppState::Launcher),
                _ => {}
            },
            AppState::InterruptConfirm(previous) => match action {
                AppAction::Confirm => self.begin_shutdown(false),
                AppAction::Back => {
                    let previous = *previous.clone();
                    self.transition(previous);
                }
                _ => {}
            },
            AppState::ResizeSuspended(resize) => {
                if resize.ready_to_resume && action == AppAction::Confirm {
                    let previous = *resize.previous.clone();
                    self.transition(previous);
                }
            }
            AppState::Shutdown(_) => {
                self.exit_requested = true;
            }
            AppState::FatalError(_) if matches!(action, AppAction::Confirm | AppAction::Back) => {
                self.begin_shutdown(true);
            }
            _ => {}
        }
    }

    pub fn handle_resize(&mut self, columns: u16, rows: u16) {
        let valid = columns >= MINIMUM_COLUMNS && rows >= MINIMUM_ROWS;
        if let AppState::ResizeSuspended(resize) = &mut self.state {
            resize.columns = columns;
            resize.rows = rows;
            resize.ready_to_resume = valid;
            self.bump_revision();
            return;
        }
        if valid || matches!(self.state, AppState::Shutdown(_)) {
            return;
        }

        let previous = Box::new(self.state.clone());
        self.transition(AppState::ResizeSuspended(ResizeSuspendedState {
            previous,
            columns,
            rows,
            ready_to_resume: false,
        }));
    }

    pub fn handle_pointer_press(&mut self, column: u16, row: u16) {
        match &mut self.state {
            AppState::Launcher if (7..=9).contains(&row) && (3..=68).contains(&column) => {
                self.transition(AppState::SoftwareDetails);
            }
            AppState::SystemMenu(menu) if (14..=17).contains(&row) => {
                menu.selected = if row <= 15 {
                    SystemMenuItem::Return
                } else {
                    SystemMenuItem::Shutdown
                };
                self.bump_revision();
            }
            _ => {}
        }
    }

    pub fn update(&mut self, _step: SimulationStep) {
        match &mut self.state {
            AppState::ColdBoot(boot) => {
                boot.elapsed_ticks = boot.elapsed_ticks.saturating_add(1);
                if boot.elapsed_ticks >= COLD_BOOT_TICKS {
                    self.transition(AppState::Launcher);
                }
            }
            AppState::WarmBoot(boot) => {
                boot.elapsed_ticks = boot.elapsed_ticks.saturating_add(1);
                if boot.elapsed_ticks >= WARM_BOOT_TICKS {
                    self.transition(AppState::Launcher);
                }
            }
            AppState::Shutdown(shutdown) => {
                shutdown.elapsed_ticks = shutdown.elapsed_ticks.saturating_add(1);
                if shutdown.urgent || shutdown.elapsed_ticks >= SHUTDOWN_TICKS {
                    self.exit_requested = true;
                }
            }
            _ => {}
        }
    }

    pub fn fail(&mut self, message: impl Into<String>) {
        self.transition(AppState::FatalError(message.into()));
    }

    pub fn render(&self, display: &mut dyn Display) -> Result<(), GlyphError> {
        crate::screens::render(self, display)
    }

    #[must_use]
    pub fn semantic_tree(&self) -> SemanticUiTree {
        let (label, children) = match &self.state {
            AppState::PrivacyNotice => (
                "Local system notice",
                vec![button("privacy.continue", "Continue", true)],
            ),
            AppState::ColdBoot(_) | AppState::WarmBoot(_) => (
                "DRX-90 system boot",
                vec![status("boot.status", "System diagnostics in progress")],
            ),
            AppState::Launcher => (
                "AfterHours software archive",
                vec![list(
                    "launcher.featured",
                    "Featured software",
                    vec![button(
                        "launcher.featured.signal-stack",
                        "Signal Stack, puzzle software released 21.11.1995",
                        true,
                    )],
                )],
            ),
            AppState::SoftwareDetails => (
                "Signal Stack software details",
                vec![button("details.return", "Return to catalog", true)],
            ),
            AppState::SystemMenu(menu) => (
                "System control",
                vec![
                    button(
                        "system.return",
                        "Return to AfterHours",
                        menu.selected == SystemMenuItem::Return,
                    ),
                    button(
                        "system.shutdown",
                        "Shut down",
                        menu.selected == SystemMenuItem::Shutdown,
                    ),
                ],
            ),
            AppState::InterruptConfirm(_) => (
                "Safe exit confirmation",
                vec![button("interrupt.confirm", "Shut down safely", true)],
            ),
            AppState::ResizeSuspended(resize) => (
                "Terminal resize required",
                vec![status(
                    "resize.status",
                    if resize.ready_to_resume {
                        "Minimum size restored. Confirm to resume."
                    } else {
                        "Terminal is smaller than 100 columns by 36 rows."
                    },
                )],
            ),
            AppState::Shutdown(_) => (
                "System shutdown",
                vec![status("shutdown.status", "Local system is shutting down")],
            ),
            AppState::FatalError(message) => (
                "Fatal application error",
                vec![status("fatal.message", message)],
            ),
        };

        SemanticUiTree {
            revision: self.semantic_revision,
            root: SemanticNode {
                id: semantic_id("application"),
                role: SemanticRole::Application,
                label: label.to_owned(),
                value: None,
                description: None,
                state: SemanticState::default(),
                actions: Vec::new(),
                children,
            },
        }
    }

    pub(crate) const fn host(&self) -> HostKind {
        self.host
    }

    pub(crate) const fn date(&self) -> CalendarDate {
        self.date
    }

    pub(crate) const fn state(&self) -> &AppState {
        &self.state
    }

    fn handle_interrupt(&mut self) {
        match &self.state {
            AppState::InterruptConfirm(_) | AppState::Shutdown(_) => {
                self.begin_shutdown(true);
            }
            _ => {
                let previous = Box::new(self.state.clone());
                self.transition(AppState::InterruptConfirm(previous));
            }
        }
    }

    fn begin_shutdown(&mut self, urgent: bool) {
        self.transition(AppState::Shutdown(ShutdownState {
            elapsed_ticks: 0,
            urgent,
        }));
        if urgent {
            self.exit_requested = true;
        }
    }

    fn transition(&mut self, next: AppState) {
        self.state = next;
        self.bump_revision();
    }

    fn bump_revision(&mut self) {
        self.semantic_revision = self.semantic_revision.saturating_add(1);
    }
}

fn semantic_id(value: &str) -> SemanticId {
    SemanticId::parse(value).expect("static semantic IDs satisfy the documented format")
}

fn button(id: &str, label: &str, focused: bool) -> SemanticNode {
    SemanticNode {
        id: semantic_id(id),
        role: SemanticRole::Button,
        label: label.to_owned(),
        value: None,
        description: None,
        state: SemanticState {
            focused,
            selected: Some(focused),
            ..SemanticState::default()
        },
        actions: vec![SemanticActionKind::Activate, SemanticActionKind::Focus],
        children: Vec::new(),
    }
}

fn status(id: &str, label: &str) -> SemanticNode {
    SemanticNode {
        id: semantic_id(id),
        role: SemanticRole::Status,
        label: label.to_owned(),
        value: None,
        description: None,
        state: SemanticState {
            live: Some(LiveRegion::Polite),
            ..SemanticState::default()
        },
        actions: Vec::new(),
        children: Vec::new(),
    }
}

fn list(id: &str, label: &str, children: Vec<SemanticNode>) -> SemanticNode {
    SemanticNode {
        id: semantic_id(id),
        role: SemanticRole::List,
        label: label.to_owned(),
        value: None,
        description: None,
        state: SemanticState::default(),
        actions: Vec::new(),
        children,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SimulationTick;

    fn app() -> Application {
        Application::new(HostKind::Native, CalendarDate::new(25, 7, 2026), false)
    }

    fn press(app: &mut Application, action: AppAction) {
        app.handle_action(action, ActionPhase::Pressed);
    }

    #[test]
    fn primary_shell_path_has_explicit_transitions() {
        let mut app = app();
        assert_eq!(app.state_kind(), AppStateKind::PrivacyNotice);

        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::ColdBoot);
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::Launcher);
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::SoftwareDetails);
        press(&mut app, AppAction::Back);
        assert_eq!(app.state_kind(), AppStateKind::Launcher);
    }

    #[test]
    fn irrelevant_actions_do_not_create_invalid_transitions() {
        let mut app = app();
        press(&mut app, AppAction::NavigateDown);
        assert_eq!(app.state_kind(), AppStateKind::PrivacyNotice);

        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::NavigateLeft);
        assert_eq!(app.state_kind(), AppStateKind::Launcher);
    }

    #[test]
    fn cold_boot_completes_after_five_seconds() {
        let mut app = Application::new(HostKind::Native, CalendarDate::new(25, 7, 2026), true);
        for tick in 1..=COLD_BOOT_TICKS {
            app.update(SimulationStep {
                tick: SimulationTick(tick),
            });
        }
        assert_eq!(app.state_kind(), AppStateKind::Launcher);
    }

    #[test]
    fn resize_requires_valid_dimensions_and_explicit_resume() {
        let mut app = Application::new(HostKind::Native, CalendarDate::new(25, 7, 2026), true);
        press(&mut app, AppAction::Confirm);
        app.handle_resize(80, 24);
        assert_eq!(app.state_kind(), AppStateKind::ResizeSuspended);
        assert!(app.is_suspended());

        app.handle_resize(100, 36);
        assert_eq!(app.state_kind(), AppStateKind::ResizeSuspended);
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::Launcher);
    }

    #[test]
    fn second_interrupt_requests_immediate_exit() {
        let mut app = app();
        press(&mut app, AppAction::Interrupt);
        assert_eq!(app.state_kind(), AppStateKind::InterruptConfirm);
        press(&mut app, AppAction::Interrupt);
        assert_eq!(app.state_kind(), AppStateKind::Shutdown);
        assert!(app.exit_requested());
    }

    #[test]
    fn semantic_focus_matches_launcher_selection() {
        let mut app = app();
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);

        let tree = app.semantic_tree();
        assert_eq!(tree.root.label, "AfterHours software archive");
        assert!(tree.root.children[0].children[0].state.focused);
    }
}
