// SPDX-License-Identifier: MPL-2.0

use crate::{
    ActionPhase, AppAction, ApplicationRepository, AssistanceProfileId, Game, GameRegistry,
    GameResult, GameStatus, InputContext, LiveRegion, NewRunRequest, RunMetadataSource,
    ScoreRecord, SemanticActionKind, SemanticId, SemanticNode, SemanticRole, SemanticState,
    SemanticUiTree, Settings, SimulationStep, StartupOptions, SystemState, TextEscapeBehavior,
    ThreeCharacterTag, insert_score, normalize_scores, ranked_scores, score_qualifies,
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
    Loading,
    Playing,
    Paused,
    Controls,
    Settings,
    GameOver,
    TagEntry,
    Scores,
    SystemMenu,
    InterruptConfirm,
    ResizeSuspended,
    Shutdown,
    FatalError,
}

#[derive(Debug)]
pub(crate) enum AppState {
    PrivacyNotice,
    PrivacyReview,
    ColdBoot(BootState),
    WarmBoot(BootState),
    Launcher,
    SoftwareDetails,
    Loading(NewRunRequest),
    Playing(GameSession),
    Paused(PauseState),
    Controls(PauseState),
    Settings(PauseState),
    GameOver(GameOverState),
    TagEntry(TagEntryState),
    Scores(ScoresState),
    SystemMenu(SystemMenuState),
    InterruptConfirm(Box<AppState>),
    ResizeSuspended(ResizeSuspendedState),
    Shutdown(ShutdownState),
    FatalError(String),
    Transitioning,
}

#[derive(Debug)]
pub(crate) struct GameSession {
    pub(crate) request: NewRunRequest,
    pub(crate) game: Box<dyn Game>,
}

#[derive(Debug)]
pub(crate) struct PauseState {
    pub(crate) session: GameSession,
    pub(crate) selected: PauseMenuItem,
    pub(crate) reason: PauseReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PauseReason {
    Player,
    FocusLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PauseMenuItem {
    Resume,
    Restart,
    Controls,
    Settings,
    Return,
    Shutdown,
}

impl PauseMenuItem {
    const ALL: [Self; 6] = [
        Self::Resume,
        Self::Restart,
        Self::Controls,
        Self::Settings,
        Self::Return,
        Self::Shutdown,
    ];
}

#[derive(Debug)]
pub(crate) struct GameOverState {
    pub(crate) session: GameSession,
    pub(crate) result: GameResult,
    pub(crate) qualifies: bool,
    pub(crate) selected: GameOverItem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameOverItem {
    Continue,
    Restart,
    Return,
}

#[derive(Debug)]
pub(crate) struct TagEntryState {
    pub(crate) session: GameSession,
    pub(crate) result: GameResult,
    pub(crate) tag: [u8; 3],
    pub(crate) cursor: usize,
}

#[derive(Debug)]
pub(crate) struct ScoresState {
    pub(crate) key: crate::ScoreRankingKey,
    pub(crate) saved: bool,
}

#[derive(Debug)]
struct RuntimeServices {
    registry: Box<dyn GameRegistry>,
    repository: Box<dyn ApplicationRepository>,
    metadata: Box<dyn RunMetadataSource>,
    settings: Settings,
    system_state: SystemState,
    scores: Vec<ScoreRecord>,
    warning: Option<String>,
    startup_options: StartupOptions,
    pending_direct: Option<NewRunRequest>,
    startup_error: Option<String>,
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
    Privacy,
    Shutdown,
}

#[derive(Debug)]
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
    runtime: Option<RuntimeServices>,
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
            runtime: None,
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
            runtime: None,
        }
    }

    #[must_use]
    pub fn with_services(
        host: HostKind,
        date: CalendarDate,
        registry: Box<dyn GameRegistry>,
        repository: Box<dyn ApplicationRepository>,
        metadata: Box<dyn RunMetadataSource>,
    ) -> Self {
        Self::with_services_and_options(
            host,
            date,
            registry,
            repository,
            metadata,
            StartupOptions::default(),
        )
    }

    #[must_use]
    pub fn with_services_and_options(
        host: HostKind,
        date: CalendarDate,
        registry: Box<dyn GameRegistry>,
        mut repository: Box<dyn ApplicationRepository>,
        mut metadata: Box<dyn RunMetadataSource>,
        startup_options: StartupOptions,
    ) -> Self {
        let (settings, settings_warning) =
            load_or_default(repository.load_settings(), Settings::default(), "settings");
        let (system_state, system_warning) = load_or_default(
            repository.load_system_state(),
            SystemState::default(),
            "system state",
        );
        let (mut scores, scores_warning) =
            load_or_default(repository.load_scores(), Vec::new(), "scores");
        let score_repair_warning = if scores_warning.is_none() && normalize_scores(&mut scores) {
            repository
                .save_scores(&scores)
                .err()
                .map(|error| format!("Repaired local scores could not be saved: {error}"))
        } else {
            None
        };
        let warning = [
            settings_warning,
            system_warning,
            scores_warning,
            score_repair_warning,
        ]
        .into_iter()
        .flatten()
        .next();
        let direct = build_direct_request(
            &startup_options,
            registry.as_ref(),
            &system_state,
            metadata.as_mut(),
        );
        let (pending_direct, startup_error) = match direct {
            Ok(request) => (request, None),
            Err(error) => (None, Some(error)),
        };
        let state = if !system_state.privacy_acknowledged {
            AppState::PrivacyNotice
        } else if let Some(error) = &startup_error {
            AppState::FatalError(error.clone())
        } else if pending_direct.is_some()
            && startup_options
                .direct_launch
                .as_ref()
                .is_some_and(|request| request.quick)
        {
            AppState::Loading(
                pending_direct
                    .clone()
                    .expect("direct request checked before startup state"),
            )
        } else if pending_direct.is_some() {
            AppState::SoftwareDetails
        } else {
            AppState::WarmBoot(BootState { elapsed_ticks: 0 })
        };
        Self {
            state,
            host,
            date,
            semantic_revision: 1,
            exit_requested: false,
            runtime: Some(RuntimeServices {
                registry,
                repository,
                metadata,
                settings,
                system_state,
                scores,
                warning,
                startup_options,
                pending_direct,
                startup_error,
            }),
        }
    }

    #[must_use]
    pub fn state_kind(&self) -> AppStateKind {
        match self.state {
            AppState::PrivacyNotice => AppStateKind::PrivacyNotice,
            AppState::PrivacyReview => AppStateKind::PrivacyNotice,
            AppState::ColdBoot(_) => AppStateKind::ColdBoot,
            AppState::WarmBoot(_) => AppStateKind::WarmBoot,
            AppState::Launcher => AppStateKind::Launcher,
            AppState::SoftwareDetails => AppStateKind::SoftwareDetails,
            AppState::Loading(_) => AppStateKind::Loading,
            AppState::Playing(_) => AppStateKind::Playing,
            AppState::Paused(_) => AppStateKind::Paused,
            AppState::Controls(_) => AppStateKind::Controls,
            AppState::Settings(_) => AppStateKind::Settings,
            AppState::GameOver(_) => AppStateKind::GameOver,
            AppState::TagEntry(_) => AppStateKind::TagEntry,
            AppState::Scores(_) => AppStateKind::Scores,
            AppState::SystemMenu(_) => AppStateKind::SystemMenu,
            AppState::InterruptConfirm(_) => AppStateKind::InterruptConfirm,
            AppState::ResizeSuspended(_) => AppStateKind::ResizeSuspended,
            AppState::Shutdown(_) => AppStateKind::Shutdown,
            AppState::FatalError(_) => AppStateKind::FatalError,
            AppState::Transitioning => {
                unreachable!("transition sentinel is never externally observable")
            }
        }
    }

    #[must_use]
    pub const fn input_context(&self) -> InputContext {
        match self.state {
            AppState::Playing(_) => InputContext::Gameplay,
            AppState::TagEntry(_) => InputContext::TextEntry(TextEscapeBehavior::Back),
            _ => InputContext::Navigation,
        }
    }

    #[must_use]
    pub const fn is_suspended(&self) -> bool {
        matches!(
            self.state,
            AppState::ResizeSuspended(_)
                | AppState::Paused(_)
                | AppState::Controls(_)
                | AppState::Settings(_)
        )
    }

    #[must_use]
    pub const fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    #[must_use]
    pub fn display_palette(&self) -> raster_display::Palette {
        self.runtime
            .as_ref()
            .map_or_else(raster_display::Palette::rcw_standard, |runtime| {
                runtime.settings.display_palette.resolve()
            })
    }

    pub fn handle_action(&mut self, action: AppAction, phase: ActionPhase) {
        if phase == ActionPhase::Released {
            return;
        }

        if action == AppAction::Interrupt {
            self.handle_interrupt();
            return;
        }

        if action == AppAction::Back && matches!(self.state, AppState::InterruptConfirm(_)) {
            let previous = match self.take_state() {
                AppState::InterruptConfirm(previous) => *previous,
                _ => unreachable!("state checked before extraction"),
            };
            self.transition(previous);
            return;
        }

        if action == AppAction::Confirm
            && matches!(
                self.state,
                AppState::ResizeSuspended(ResizeSuspendedState {
                    ready_to_resume: true,
                    ..
                })
            )
        {
            let previous = match self.take_state() {
                AppState::ResizeSuspended(resize) => *resize.previous,
                _ => unreachable!("state checked before extraction"),
            };
            self.transition(previous);
            return;
        }

        if matches!(self.state, AppState::Playing(_)) {
            match action {
                AppAction::Pause | AppAction::Back => self.pause(PauseReason::Player),
                AppAction::Game(game_action) => {
                    if let AppState::Playing(session) = &mut self.state
                        && let Err(error) = session.game.handle_action(game_action, phase)
                    {
                        self.fail(error.to_string());
                    }
                }
                _ => {}
            }
            return;
        }
        if matches!(self.state, AppState::Paused(_)) {
            self.handle_pause_action(action);
            return;
        }
        if matches!(self.state, AppState::Controls(_) | AppState::Settings(_)) {
            self.handle_pause_subscreen_action(action);
            return;
        }
        if matches!(self.state, AppState::GameOver(_)) {
            self.handle_game_over_action(action);
            return;
        }
        if matches!(self.state, AppState::TagEntry(_)) {
            self.handle_tag_action(action);
            return;
        }
        if matches!(self.state, AppState::Scores(_)) {
            self.handle_scores_action(action);
            return;
        }

        match &mut self.state {
            AppState::PrivacyNotice if action == AppAction::Confirm => {
                self.acknowledge_privacy();
                let next = self.post_privacy_state();
                self.transition(next);
            }
            AppState::PrivacyReview if matches!(action, AppAction::Confirm | AppAction::Back) => {
                self.transition(AppState::Launcher);
            }
            AppState::ColdBoot(_) | AppState::WarmBoot(_) => {
                self.transition(AppState::Launcher);
            }
            AppState::Launcher => match action {
                AppAction::Confirm => self.transition(AppState::SoftwareDetails),
                AppAction::OpenScores => self.transition(AppState::Scores(ScoresState {
                    key: signal_stack_ranking_key(),
                    saved: false,
                })),
                AppAction::Back => self.transition(AppState::SystemMenu(SystemMenuState {
                    selected: SystemMenuItem::Return,
                })),
                _ => {}
            },
            AppState::SoftwareDetails => match action {
                AppAction::Confirm => self.begin_run(),
                AppAction::Back => self.transition(AppState::Launcher),
                _ => {}
            },
            AppState::SystemMenu(menu) => match action {
                AppAction::NavigateUp | AppAction::NavigateDown => {
                    menu.selected = match (menu.selected, action) {
                        (SystemMenuItem::Return, AppAction::NavigateUp) => SystemMenuItem::Shutdown,
                        (SystemMenuItem::Return, _) => SystemMenuItem::Privacy,
                        (SystemMenuItem::Privacy, AppAction::NavigateUp) => SystemMenuItem::Return,
                        (SystemMenuItem::Privacy, _) => SystemMenuItem::Shutdown,
                        (SystemMenuItem::Shutdown, AppAction::NavigateUp) => {
                            SystemMenuItem::Privacy
                        }
                        (SystemMenuItem::Shutdown, _) => SystemMenuItem::Return,
                    };
                    self.bump_revision();
                }
                AppAction::Confirm => match menu.selected {
                    SystemMenuItem::Return => self.transition(AppState::Launcher),
                    SystemMenuItem::Privacy => self.transition(AppState::PrivacyReview),
                    SystemMenuItem::Shutdown => self.begin_shutdown(false),
                },
                AppAction::Back => self.transition(AppState::Launcher),
                _ => {}
            },
            AppState::InterruptConfirm(_) => match action {
                AppAction::Confirm => self.begin_shutdown(false),
                AppAction::Back => unreachable!("handled before borrowing state"),
                _ => {}
            },
            AppState::ResizeSuspended(_) => {}
            AppState::Shutdown(_) => {
                self.exit_requested = true;
            }
            AppState::FatalError(_) if matches!(action, AppAction::Confirm | AppAction::Back) => {
                self.begin_shutdown(true);
            }
            AppState::Loading(_)
            | AppState::Playing(_)
            | AppState::Paused(_)
            | AppState::Controls(_)
            | AppState::Settings(_)
            | AppState::GameOver(_)
            | AppState::TagEntry(_)
            | AppState::Scores(_) => {}
            AppState::Transitioning => {
                unreachable!("transition sentinel is never externally observable")
            }
            _ => {}
        }
    }

    /// Skips an active boot sequence and reports whether the triggering host
    /// event was consumed.
    pub fn handle_activity(&mut self) -> bool {
        if matches!(self.state, AppState::ColdBoot(_) | AppState::WarmBoot(_)) {
            self.transition(AppState::Launcher);
            true
        } else {
            false
        }
    }

    pub fn handle_focus_lost(&mut self) {
        if pause_nested_game(&mut self.state) {
            self.bump_revision();
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

        let previous = Box::new(self.take_state());
        self.transition(AppState::ResizeSuspended(ResizeSuspendedState {
            previous,
            columns,
            rows,
            ready_to_resume: false,
        }));
    }

    pub fn handle_pointer_press(&mut self, column: u16, row: u16) {
        if matches!(self.state, AppState::ColdBoot(_) | AppState::WarmBoot(_)) {
            self.transition(AppState::Launcher);
            return;
        }
        match &mut self.state {
            AppState::Launcher if (7..=9).contains(&row) && (3..=68).contains(&column) => {
                self.transition(AppState::SoftwareDetails);
            }
            AppState::SystemMenu(menu) if (14..=20).contains(&row) => {
                menu.selected = match row {
                    14..=16 => SystemMenuItem::Return,
                    17..=18 => SystemMenuItem::Privacy,
                    _ => SystemMenuItem::Shutdown,
                };
                self.bump_revision();
            }
            _ => {}
        }
    }

    pub fn activate_semantic_node(&mut self, id: &SemanticId) {
        match id.as_str() {
            "privacy.continue" | "launcher.featured.signal-stack" | "details.start" => {
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "details.return" => {
                self.handle_action(AppAction::Back, ActionPhase::Pressed);
            }
            "privacy.return" => self.transition(AppState::Launcher),
            "system.return" => self.transition(AppState::Launcher),
            "system.privacy" => self.transition(AppState::PrivacyReview),
            "system.shutdown" | "interrupt.confirm" => self.begin_shutdown(false),
            "pause.resume" => {
                if let AppState::Paused(pause) = &mut self.state {
                    pause.selected = PauseMenuItem::Resume;
                }
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "pause.restart" => {
                if let AppState::Paused(pause) = &mut self.state {
                    pause.selected = PauseMenuItem::Restart;
                }
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "pause.controls" => {
                if let AppState::Paused(pause) = &mut self.state {
                    pause.selected = PauseMenuItem::Controls;
                }
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "pause.settings" => {
                if let AppState::Paused(pause) = &mut self.state {
                    pause.selected = PauseMenuItem::Settings;
                }
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "pause.return" => {
                if let AppState::Paused(pause) = &mut self.state {
                    pause.selected = PauseMenuItem::Return;
                }
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "pause.shutdown" => {
                if let AppState::Paused(pause) = &mut self.state {
                    pause.selected = PauseMenuItem::Shutdown;
                }
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "controls.return" => {
                self.handle_action(AppAction::Back, ActionPhase::Pressed);
            }
            "settings.reduced-motion" => {
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "game-over.continue" | "game-over.restart" | "game-over.return" => {
                if let AppState::GameOver(game_over) = &mut self.state {
                    game_over.selected = match id.as_str() {
                        "game-over.continue" => GameOverItem::Continue,
                        "game-over.restart" => GameOverItem::Restart,
                        "game-over.return" => GameOverItem::Return,
                        _ => unreachable!("semantic ID matched above"),
                    };
                }
                self.handle_action(AppAction::Confirm, ActionPhase::Pressed);
            }
            "tag.character-0" | "tag.character-1" | "tag.character-2" => {
                if let AppState::TagEntry(tag) = &mut self.state {
                    let index = usize::from(id.as_str().as_bytes()[14] - b'0');
                    tag.cursor = index;
                    tag.tag[index] = cycle_tag_character(tag.tag[index], true);
                    self.bump_revision();
                }
            }
            "tag.submit" => self.handle_action(AppAction::Confirm, ActionPhase::Pressed),
            "tag.back" => self.handle_action(AppAction::Back, ActionPhase::Pressed),
            "scores.return" => self.transition(AppState::Launcher),
            _ => {}
        }
    }

    pub fn update(&mut self, step: SimulationStep) {
        if matches!(self.state, AppState::Loading(_)) {
            self.finish_loading();
            return;
        }
        if matches!(self.state, AppState::Playing(_)) {
            let outcome = if let AppState::Playing(session) = &mut self.state {
                session
                    .game
                    .update(step)
                    .map(|()| (session.game.status(), session.game.result()))
            } else {
                unreachable!("state checked before update")
            };
            match outcome {
                Err(error) => self.fail(error.to_string()),
                Ok((GameStatus::Finished, Some(result))) => self.finish_run(result),
                Ok((GameStatus::Finished, None)) => {
                    self.fail("game finished without a result envelope");
                }
                Ok(_) => {}
            }
            return;
        }

        let cold_boot_limit = self.boot_tick_limit(false);
        let warm_boot_limit = self.boot_tick_limit(true);
        match &mut self.state {
            AppState::ColdBoot(boot) => {
                boot.elapsed_ticks = boot.elapsed_ticks.saturating_add(1);
                if boot.elapsed_ticks >= cold_boot_limit {
                    self.transition(AppState::Launcher);
                }
            }
            AppState::WarmBoot(boot) => {
                boot.elapsed_ticks = boot.elapsed_ticks.saturating_add(1);
                if boot.elapsed_ticks >= warm_boot_limit {
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

    pub fn report_persistence_unavailable(&mut self, message: impl Into<String>) {
        if let Some(runtime) = &mut self.runtime {
            runtime.warning = Some(message.into());
            self.bump_revision();
        }
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
            AppState::PrivacyReview => (
                "Local system notice",
                vec![button("privacy.return", "Return to AfterHours", true)],
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
                vec![
                    button("details.start", "Start Standard Transmission", true),
                    button("details.return", "Return to catalog", false),
                ],
            ),
            AppState::Loading(_) => (
                "Signal Stack loading",
                vec![status("game.loading", "Loading Standard Transmission")],
            ),
            AppState::Playing(_) => (
                "Signal Stack gameplay",
                vec![status("game.status", "Transmission active")],
            ),
            AppState::Paused(pause) => (
                "Signal Stack paused",
                PauseMenuItem::ALL
                    .iter()
                    .map(|item| {
                        button(
                            pause_semantic_id(*item),
                            pause_label(*item),
                            pause.selected == *item,
                        )
                    })
                    .collect(),
            ),
            AppState::Controls(_) => (
                "Signal Stack controls",
                vec![button("controls.return", "Return to pause menu", true)],
            ),
            AppState::Settings(_) => (
                "Accessibility settings",
                vec![button(
                    "settings.reduced-motion",
                    if self.reduced_motion() {
                        "Reduced motion enabled"
                    } else {
                        "Reduced motion disabled"
                    },
                    true,
                )],
            ),
            AppState::GameOver(game_over) => (
                "Signal Stack transmission terminated",
                vec![
                    status(
                        "game-over.score",
                        &format!("Final score {}", game_over.result.score),
                    ),
                    button(
                        "game-over.continue",
                        if game_over.qualifies {
                            "Enter operator tag"
                        } else {
                            "View local records"
                        },
                        game_over.selected == GameOverItem::Continue,
                    ),
                    button(
                        "game-over.restart",
                        "Restart",
                        game_over.selected == GameOverItem::Restart,
                    ),
                    button(
                        "game-over.return",
                        "Return to AfterHours",
                        game_over.selected == GameOverItem::Return,
                    ),
                ],
            ),
            AppState::TagEntry(tag) => (
                "Operator identification entry",
                vec![
                    status(
                        "tag.value",
                        std::str::from_utf8(&tag.tag).expect("tag editor is ASCII"),
                    ),
                    button(
                        "tag.character-0",
                        &format!("First character: {}", char::from(tag.tag[0])),
                        tag.cursor == 0,
                    ),
                    button(
                        "tag.character-1",
                        &format!("Second character: {}", char::from(tag.tag[1])),
                        tag.cursor == 1,
                    ),
                    button(
                        "tag.character-2",
                        &format!("Third character: {}", char::from(tag.tag[2])),
                        tag.cursor == 2,
                    ),
                    button("tag.submit", "Submit operator tag", false),
                    button("tag.back", "Return to diagnostics", false),
                ],
            ),
            AppState::Scores(scores) => (
                "Signal Stack local records",
                vec![
                    status(
                        "scores.status",
                        if scores.saved {
                            "Score saved locally"
                        } else {
                            "Local score table"
                        },
                    ),
                    button("scores.return", "Return to AfterHours", true),
                ],
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
                        "system.privacy",
                        "Local system notice",
                        menu.selected == SystemMenuItem::Privacy,
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
            AppState::Transitioning => {
                unreachable!("transition sentinel is never externally observable")
            }
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
                let previous = Box::new(self.take_state());
                self.transition(AppState::InterruptConfirm(previous));
            }
        }
    }

    fn acknowledge_privacy(&mut self) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };
        runtime.system_state.privacy_acknowledged = true;
        if let Err(error) = runtime.repository.save_system_state(&runtime.system_state) {
            runtime.warning = Some(format!("Privacy preference was not saved: {error}"));
        }
    }

    fn begin_run(&mut self) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };
        if let Some(request) = runtime.pending_direct.take() {
            remember_run(runtime, &request);
            self.transition(AppState::Loading(request));
            return;
        }
        let Some(descriptor) = runtime
            .registry
            .advertised_descriptors()
            .into_iter()
            .find(|descriptor| descriptor.id.as_str() == "signal-stack")
        else {
            self.fail("Signal Stack is not installed");
            return;
        };
        let Some(mode) = descriptor.modes.first() else {
            self.fail("Signal Stack has no playable mode");
            return;
        };
        let request = NewRunRequest {
            game_id: descriptor.id.clone(),
            mode_id: mode.id.clone(),
            rules_revision: descriptor.rules_revision,
            seed: runtime.metadata.next_seed(),
        };
        remember_run(runtime, &request);
        self.transition(AppState::Loading(request));
    }

    fn finish_loading(&mut self) {
        let request = match self.take_state() {
            AppState::Loading(request) => request,
            _ => unreachable!("state checked before extraction"),
        };
        let result = self.runtime.as_mut().map_or_else(
            || {
                Err(crate::GameError::Runtime(
                    "game runtime is unavailable".into(),
                ))
            },
            |runtime| {
                let mut game = runtime.registry.create(&request.game_id)?;
                game.reset(&request)?;
                Ok(game)
            },
        );
        match result {
            Ok(game) => self.transition(AppState::Playing(GameSession { request, game })),
            Err(error) => self.fail(error.to_string()),
        }
    }

    fn pause(&mut self, reason: PauseReason) {
        let mut session = match self.take_state() {
            AppState::Playing(session) => session,
            _ => unreachable!("state checked before extraction"),
        };
        session.game.set_paused(true);
        self.transition(AppState::Paused(PauseState {
            session,
            selected: PauseMenuItem::Resume,
            reason,
        }));
    }

    fn handle_pause_action(&mut self, action: AppAction) {
        if matches!(action, AppAction::Back | AppAction::Pause) {
            self.resume();
            return;
        }
        if matches!(action, AppAction::NavigateUp | AppAction::NavigateDown) {
            if let AppState::Paused(pause) = &mut self.state {
                let current = PauseMenuItem::ALL
                    .iter()
                    .position(|item| *item == pause.selected)
                    .expect("pause item belongs to menu");
                let next = if action == AppAction::NavigateUp {
                    current
                        .checked_sub(1)
                        .unwrap_or(PauseMenuItem::ALL.len() - 1)
                } else {
                    (current + 1) % PauseMenuItem::ALL.len()
                };
                pause.selected = PauseMenuItem::ALL[next];
                self.bump_revision();
            }
            return;
        }
        if action != AppAction::Confirm {
            return;
        }

        let pause = match self.take_state() {
            AppState::Paused(pause) => pause,
            _ => unreachable!("state checked before extraction"),
        };
        match pause.selected {
            PauseMenuItem::Resume => self.resume_from(pause),
            PauseMenuItem::Restart => self.restart_session(pause.session),
            PauseMenuItem::Controls => self.transition(AppState::Controls(pause)),
            PauseMenuItem::Settings => self.transition(AppState::Settings(pause)),
            PauseMenuItem::Return => self.transition(AppState::Launcher),
            PauseMenuItem::Shutdown => self.begin_shutdown(false),
        }
    }

    fn handle_pause_subscreen_action(&mut self, action: AppAction) {
        if matches!(self.state, AppState::Settings(_)) && action == AppAction::Confirm {
            if let Some(runtime) = &mut self.runtime {
                runtime.settings.reduced_motion = !runtime.settings.reduced_motion;
                if let Err(error) = runtime.repository.save_settings(&runtime.settings) {
                    runtime.warning = Some(format!("Settings were not saved: {error}"));
                }
            }
            self.bump_revision();
            return;
        }
        if !matches!(action, AppAction::Back | AppAction::Pause) {
            return;
        }
        let pause = match self.take_state() {
            AppState::Controls(pause) | AppState::Settings(pause) => pause,
            _ => unreachable!("state checked before extraction"),
        };
        self.transition(AppState::Paused(pause));
    }

    fn resume(&mut self) {
        let pause = match self.take_state() {
            AppState::Paused(pause) => pause,
            _ => unreachable!("state checked before extraction"),
        };
        self.resume_from(pause);
    }

    fn resume_from(&mut self, mut pause: PauseState) {
        pause.session.game.set_paused(false);
        self.transition(AppState::Playing(pause.session));
    }

    fn restart_session(&mut self, session: GameSession) {
        let Some(runtime) = &mut self.runtime else {
            self.transition(AppState::Launcher);
            return;
        };
        let request = NewRunRequest {
            seed: runtime.metadata.next_seed(),
            ..session.request
        };
        self.transition(AppState::Loading(request));
    }

    fn finish_run(&mut self, result: GameResult) {
        let session = match self.take_state() {
            AppState::Playing(session) => session,
            _ => unreachable!("state checked before extraction"),
        };
        if result.game_id != session.request.game_id
            || result.mode_id != session.request.mode_id
            || result.rules_revision != session.request.rules_revision
            || result.seed != session.request.seed
        {
            self.fail("game returned a result that does not match the active run");
            return;
        }
        if result
            .discoveries
            .contains(&crate::DiscoveryMarker::PacketSweepTrace)
            && let Some(runtime) = &mut self.runtime
            && !runtime.system_state.packet_sweep_trace_revealed
        {
            runtime.system_state.packet_sweep_trace_revealed = true;
            if let Err(error) = runtime.repository.save_system_state(&runtime.system_state) {
                runtime.warning = Some(format!("Trace discovery was not remembered: {error}"));
            }
        }
        let key = crate::ScoreRankingKey {
            game_id: result.game_id.clone(),
            mode_id: result.mode_id.clone(),
            rules_revision: result.rules_revision,
            assistance_profile: canonical_assistance(),
        };
        let qualifies = self
            .runtime
            .as_ref()
            .is_some_and(|runtime| score_qualifies(&runtime.scores, &key, result.score));
        self.transition(AppState::GameOver(GameOverState {
            session,
            result,
            qualifies,
            selected: GameOverItem::Continue,
        }));
    }

    fn handle_game_over_action(&mut self, action: AppAction) {
        if matches!(action, AppAction::NavigateLeft | AppAction::NavigateRight) {
            if let AppState::GameOver(game_over) = &mut self.state {
                game_over.selected = match (game_over.selected, action) {
                    (GameOverItem::Continue, AppAction::NavigateLeft) => GameOverItem::Return,
                    (GameOverItem::Continue, _) => GameOverItem::Restart,
                    (GameOverItem::Restart, AppAction::NavigateLeft) => GameOverItem::Continue,
                    (GameOverItem::Restart, _) => GameOverItem::Return,
                    (GameOverItem::Return, AppAction::NavigateLeft) => GameOverItem::Restart,
                    (GameOverItem::Return, _) => GameOverItem::Continue,
                };
                self.bump_revision();
            }
            return;
        }
        if action == AppAction::Back {
            self.transition(AppState::Launcher);
            return;
        }
        if action != AppAction::Confirm {
            return;
        }
        let game_over = match self.take_state() {
            AppState::GameOver(game_over) => game_over,
            _ => unreachable!("state checked before extraction"),
        };
        match game_over.selected {
            GameOverItem::Continue if game_over.qualifies => {
                let previous = self
                    .runtime
                    .as_ref()
                    .and_then(|runtime| runtime.system_state.last_score_tag)
                    .unwrap_or_else(default_tag);
                self.transition(AppState::TagEntry(TagEntryState {
                    session: game_over.session,
                    result: game_over.result,
                    tag: previous
                        .as_str()
                        .as_bytes()
                        .try_into()
                        .expect("tag has 3 bytes"),
                    cursor: 0,
                }));
            }
            GameOverItem::Continue => {
                let key = result_key(&game_over.result);
                self.transition(AppState::Scores(ScoresState { key, saved: false }));
            }
            GameOverItem::Restart => self.restart_session(game_over.session),
            GameOverItem::Return => self.transition(AppState::Launcher),
        }
    }

    fn handle_tag_action(&mut self, action: AppAction) {
        if action == AppAction::Back {
            let tag = match self.take_state() {
                AppState::TagEntry(tag) => tag,
                _ => unreachable!("state checked before extraction"),
            };
            self.transition(AppState::GameOver(GameOverState {
                session: tag.session,
                result: tag.result,
                qualifies: true,
                selected: GameOverItem::Continue,
            }));
            return;
        }
        if action == AppAction::Confirm {
            self.submit_tag();
            return;
        }
        if let AppState::TagEntry(tag) = &mut self.state {
            match action {
                AppAction::NavigateLeft => tag.cursor = tag.cursor.saturating_sub(1),
                AppAction::NavigateRight => tag.cursor = (tag.cursor + 1).min(2),
                AppAction::NavigateUp | AppAction::NavigateDown => {
                    tag.tag[tag.cursor] =
                        cycle_tag_character(tag.tag[tag.cursor], action == AppAction::NavigateUp);
                }
                AppAction::DeleteBackward => {
                    tag.tag[tag.cursor] = b'-';
                    tag.cursor = tag.cursor.saturating_sub(1);
                }
                AppAction::DeleteForward => tag.tag[tag.cursor] = b'-',
                AppAction::TextInput(character) if valid_tag_character(character) => {
                    tag.tag[tag.cursor] = character.to_ascii_uppercase() as u8;
                    tag.cursor = (tag.cursor + 1).min(2);
                }
                _ => return,
            }
            self.bump_revision();
        }
    }

    fn submit_tag(&mut self) {
        let tag_entry = match self.take_state() {
            AppState::TagEntry(tag) => tag,
            _ => unreachable!("state checked before extraction"),
        };
        let tag_text = std::str::from_utf8(&tag_entry.tag).expect("tag editor is ASCII");
        let Ok(tag) = ThreeCharacterTag::parse(tag_text) else {
            self.transition(AppState::TagEntry(tag_entry));
            return;
        };
        let key = result_key(&tag_entry.result);
        let mut saved = false;
        if let Some(runtime) = &mut self.runtime {
            let record = ScoreRecord {
                game_id: tag_entry.result.game_id.clone(),
                mode_id: tag_entry.result.mode_id.clone(),
                rules_revision: tag_entry.result.rules_revision,
                assistance_profile: canonical_assistance(),
                tag,
                score: tag_entry.result.score,
                duration: tag_entry.result.final_tick,
                seed: tag_entry.result.seed,
                outcome: tag_entry.result.outcome,
                final_state_hash: tag_entry.result.final_state_hash,
                recorded_at_unix_seconds: runtime.metadata.unix_seconds(),
            };
            if insert_score(&mut runtime.scores, record) {
                saved = runtime.repository.save_scores(&runtime.scores).is_ok();
                if !saved {
                    runtime.warning =
                        Some("Score remains in memory but could not be saved.".to_owned());
                }
            }
            runtime.system_state.last_score_tag = Some(tag);
            if let Err(error) = runtime.repository.save_system_state(&runtime.system_state) {
                runtime.warning = Some(format!("Operator tag was not remembered: {error}"));
            }
        }
        self.transition(AppState::Scores(ScoresState { key, saved }));
    }

    fn handle_scores_action(&mut self, action: AppAction) {
        match action {
            AppAction::Back | AppAction::Confirm => self.transition(AppState::Launcher),
            _ => {}
        }
    }

    pub(crate) fn ranked_score_rows(&self, key: &crate::ScoreRankingKey) -> Vec<&ScoreRecord> {
        self.runtime
            .as_ref()
            .map_or_else(Vec::new, |runtime| ranked_scores(&runtime.scores, key))
    }

    pub(crate) fn persistence_warning(&self) -> Option<&str> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.warning.as_deref())
    }

    pub(crate) fn reduced_motion(&self) -> bool {
        self.runtime
            .as_ref()
            .is_some_and(|runtime| runtime.settings.reduced_motion)
    }

    fn post_privacy_state(&self) -> AppState {
        let Some(runtime) = &self.runtime else {
            return AppState::ColdBoot(BootState { elapsed_ticks: 0 });
        };
        if let Some(error) = &runtime.startup_error {
            AppState::FatalError(error.clone())
        } else if let Some(request) = &runtime.pending_direct {
            if runtime
                .startup_options
                .direct_launch
                .as_ref()
                .is_some_and(|direct| direct.quick)
            {
                AppState::Loading(request.clone())
            } else {
                AppState::SoftwareDetails
            }
        } else {
            AppState::ColdBoot(BootState { elapsed_ticks: 0 })
        }
    }

    fn boot_tick_limit(&self, warm: bool) -> u64 {
        let quiet = self.runtime.as_ref().is_some_and(|runtime| {
            runtime.startup_options.quiet || runtime.settings.quiet_operation
        });
        match (warm, quiet) {
            (true, true) => 30,
            (false, true) => 60,
            (true, false) => WARM_BOOT_TICKS,
            (false, false) => COLD_BOOT_TICKS,
        }
    }

    pub(crate) fn best_signal_stack_score(&self) -> Option<u64> {
        self.runtime.as_ref().and_then(|runtime| {
            ranked_scores(&runtime.scores, &signal_stack_ranking_key())
                .first()
                .map(|record| record.score)
        })
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

    fn take_state(&mut self) -> AppState {
        std::mem::replace(&mut self.state, AppState::Transitioning)
    }

    fn bump_revision(&mut self) {
        self.semantic_revision = self.semantic_revision.saturating_add(1);
    }
}

fn build_direct_request(
    options: &StartupOptions,
    registry: &dyn GameRegistry,
    system_state: &SystemState,
    metadata: &mut dyn RunMetadataSource,
) -> Result<Option<NewRunRequest>, String> {
    let Some(direct) = &options.direct_launch else {
        return Ok(None);
    };
    let advertised = registry.advertised_descriptors();
    let hidden = registry.hidden_descriptors();
    let descriptor = advertised
        .iter()
        .chain(hidden.iter())
        .find(|descriptor| descriptor.id == direct.game_id)
        .ok_or_else(|| format!("Requested game {} is not installed.", direct.game_id))?;
    if descriptor.visibility == crate::CatalogVisibility::Hidden
        && !(descriptor.id.as_str() == "packet-sweep" && system_state.packet_sweep_unlocked)
    {
        return Err(format!(
            "Requested game {} is not available for direct launch.",
            descriptor.id
        ));
    }
    let mode = descriptor
        .modes
        .first()
        .ok_or_else(|| format!("Requested game {} has no playable mode.", descriptor.id))?;
    Ok(Some(NewRunRequest {
        game_id: descriptor.id.clone(),
        mode_id: mode.id.clone(),
        rules_revision: descriptor.rules_revision,
        seed: direct.seed.unwrap_or_else(|| metadata.next_seed()),
    }))
}

fn remember_run(runtime: &mut RuntimeServices, request: &NewRunRequest) {
    runtime.system_state.last_selected_game = Some(request.game_id.clone());
    runtime.system_state.last_game_mode = Some(request.mode_id.clone());
    if let Err(error) = runtime.repository.save_system_state(&runtime.system_state) {
        runtime.warning = Some(format!("Launcher state was not remembered: {error}"));
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

fn load_or_default<T>(
    result: Result<T, crate::PersistenceError>,
    default: T,
    label: &str,
) -> (T, Option<String>) {
    match result {
        Ok(value) => (value, None),
        Err(error) => (
            default,
            Some(format!("Stored {label} could not be loaded: {error}")),
        ),
    }
}

fn canonical_assistance() -> AssistanceProfileId {
    AssistanceProfileId::parse("canonical").expect("static assistance profile is valid")
}

fn result_key(result: &GameResult) -> crate::ScoreRankingKey {
    crate::ScoreRankingKey {
        game_id: result.game_id.clone(),
        mode_id: result.mode_id.clone(),
        rules_revision: result.rules_revision,
        assistance_profile: canonical_assistance(),
    }
}

fn signal_stack_ranking_key() -> crate::ScoreRankingKey {
    crate::ScoreRankingKey {
        game_id: crate::GameId::parse("signal-stack").expect("static game ID is valid"),
        mode_id: crate::ModeId::parse("standard-transmission").expect("static mode ID is valid"),
        rules_revision: crate::RulesRevision::new(1).expect("static rules revision is valid"),
        assistance_profile: canonical_assistance(),
    }
}

fn default_tag() -> ThreeCharacterTag {
    ThreeCharacterTag::parse("---").expect("default score tag is valid")
}

fn valid_tag_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '.' | '_')
}

fn cycle_tag_character(character: u8, forward: bool) -> u8 {
    const CHARACTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._";
    let index = CHARACTERS
        .iter()
        .position(|candidate| *candidate == character)
        .unwrap_or(0);
    if forward {
        CHARACTERS[(index + 1) % CHARACTERS.len()]
    } else {
        CHARACTERS[index.checked_sub(1).unwrap_or(CHARACTERS.len() - 1)]
    }
}

const fn pause_label(item: PauseMenuItem) -> &'static str {
    match item {
        PauseMenuItem::Resume => "Resume",
        PauseMenuItem::Restart => "Restart",
        PauseMenuItem::Controls => "Controls",
        PauseMenuItem::Settings => "Settings",
        PauseMenuItem::Return => "Return to AfterHours",
        PauseMenuItem::Shutdown => "Shut down",
    }
}

const fn pause_semantic_id(item: PauseMenuItem) -> &'static str {
    match item {
        PauseMenuItem::Resume => "pause.resume",
        PauseMenuItem::Restart => "pause.restart",
        PauseMenuItem::Controls => "pause.controls",
        PauseMenuItem::Settings => "pause.settings",
        PauseMenuItem::Return => "pause.return",
        PauseMenuItem::Shutdown => "pause.shutdown",
    }
}

fn pause_nested_game(state: &mut AppState) -> bool {
    match state {
        AppState::Playing(_) => {
            let previous = std::mem::replace(state, AppState::Transitioning);
            let AppState::Playing(mut session) = previous else {
                unreachable!("state matched before extraction");
            };
            session.game.set_paused(true);
            *state = AppState::Paused(PauseState {
                session,
                selected: PauseMenuItem::Resume,
                reason: PauseReason::FocusLost,
            });
            true
        }
        AppState::ResizeSuspended(resize) => pause_nested_game(&mut resize.previous),
        AppState::InterruptConfirm(previous) => pause_nested_game(previous),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::{
        GameAction, GameDescriptor, GameError, GameId, GameOutcome, ModeDescriptor, ModeId,
        PersistenceError, RulesRevision, RunSeed, ScoreRepository, SettingsRepository,
        SimulationTick, StateHash, SystemStateRepository,
    };
    use raster_display::{DISPLAY_SIZE, Display};

    fn app() -> Application {
        Application::new(HostKind::Native, CalendarDate::new(25, 7, 2026), false)
    }

    fn press(app: &mut Application, action: AppAction) {
        app.handle_action(action, ActionPhase::Pressed);
    }

    fn serviced_app() -> Application {
        Application::with_services(
            HostKind::Native,
            CalendarDate::new(25, 7, 2026),
            Box::new(TestRegistry),
            Box::new(TestRepository::default()),
            Box::new(TestMetadata),
        )
    }

    fn stored_score(score: u64, recorded_at_unix_seconds: i64) -> ScoreRecord {
        ScoreRecord {
            game_id: GameId::parse("signal-stack").expect("valid ID"),
            mode_id: ModeId::parse("standard-transmission").expect("valid ID"),
            rules_revision: RulesRevision::new(1).expect("valid revision"),
            assistance_profile: canonical_assistance(),
            tag: ThreeCharacterTag::parse("NUL").expect("valid tag"),
            score,
            duration: SimulationTick(60),
            seed: RunSeed(1),
            outcome: GameOutcome::GameOver,
            final_state_hash: StateHash(score),
            recorded_at_unix_seconds,
        }
    }

    #[test]
    fn startup_durably_repairs_an_oversized_score_board() {
        let scores = Arc::new(Mutex::new(
            (0..crate::LOCAL_SCORE_LIMIT + 3)
                .map(|index| stored_score(1_000 - index as u64, index as i64))
                .collect(),
        ));

        let _app = Application::with_services(
            HostKind::Native,
            CalendarDate::new(25, 7, 2026),
            Box::new(TestRegistry),
            Box::new(SharedScoreRepository {
                scores: Arc::clone(&scores),
            }),
            Box::new(TestMetadata),
        );

        let persisted = scores.lock().expect("score lock");
        assert_eq!(persisted.len(), crate::LOCAL_SCORE_LIMIT);
        assert!(persisted.iter().all(|record| record.score >= 991));
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
    fn quick_direct_launch_never_skips_privacy() {
        let mut app = Application::with_services_and_options(
            HostKind::Native,
            CalendarDate::new(25, 7, 2026),
            Box::new(TestRegistry),
            Box::new(TestRepository::default()),
            Box::new(TestMetadata),
            StartupOptions {
                quiet: false,
                direct_launch: Some(crate::DirectLaunchRequest {
                    game_id: GameId::parse("signal-stack").expect("ID"),
                    quick: true,
                    seed: Some(RunSeed(42)),
                }),
            },
        );

        assert_eq!(app.state_kind(), AppStateKind::PrivacyNotice);
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::Loading);
        app.update(SimulationStep {
            tick: SimulationTick(1),
        });
        assert_eq!(app.state_kind(), AppStateKind::Playing);
    }

    #[test]
    fn normal_direct_launch_shows_details_before_loading() {
        let repository = TestRepository {
            system: SystemState {
                privacy_acknowledged: true,
                ..SystemState::default()
            },
            ..TestRepository::default()
        };
        let mut app = Application::with_services_and_options(
            HostKind::Native,
            CalendarDate::new(25, 7, 2026),
            Box::new(TestRegistry),
            Box::new(repository),
            Box::new(TestMetadata),
            StartupOptions {
                quiet: false,
                direct_launch: Some(crate::DirectLaunchRequest {
                    game_id: GameId::parse("signal-stack").expect("ID"),
                    quick: false,
                    seed: None,
                }),
            },
        );

        assert_eq!(app.state_kind(), AppStateKind::SoftwareDetails);
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::Loading);
    }

    #[test]
    fn command_line_quiet_shortens_boot_without_changing_saved_setting() {
        let repository = TestRepository {
            system: SystemState {
                privacy_acknowledged: true,
                ..SystemState::default()
            },
            ..TestRepository::default()
        };
        let mut app = Application::with_services_and_options(
            HostKind::Native,
            CalendarDate::new(25, 7, 2026),
            Box::new(TestRegistry),
            Box::new(repository),
            Box::new(TestMetadata),
            StartupOptions {
                quiet: true,
                direct_launch: None,
            },
        );

        for tick in 1..=30 {
            app.update(SimulationStep {
                tick: SimulationTick(tick),
            });
        }

        assert_eq!(app.state_kind(), AppStateKind::Launcher);
        assert!(
            !app.runtime
                .as_ref()
                .expect("runtime")
                .settings
                .quiet_operation
        );
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

    #[test]
    fn semantic_activation_uses_the_same_validated_transitions() {
        let mut app = app();
        app.activate_semantic_node(
            &SemanticId::parse("privacy.continue").expect("test ID is valid"),
        );
        assert_eq!(app.state_kind(), AppStateKind::ColdBoot);

        press(&mut app, AppAction::Confirm);
        app.activate_semantic_node(
            &SemanticId::parse("launcher.featured.signal-stack").expect("test ID is valid"),
        );
        assert_eq!(app.state_kind(), AppStateKind::SoftwareDetails);
    }

    #[test]
    fn complete_gameplay_loop_reaches_persisted_scores() {
        let mut app = serviced_app();
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::SoftwareDetails);

        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::Loading);
        app.update(SimulationStep {
            tick: SimulationTick(1),
        });
        assert_eq!(app.state_kind(), AppStateKind::Playing);

        press(&mut app, AppAction::Pause);
        assert_eq!(app.state_kind(), AppStateKind::Paused);
        assert!(app.is_suspended());
        let mut display = raster_display::DisplayBuffer::canonical();
        app.render(&mut display).expect("pause screen renders");
        let snapshot = display.snapshot();
        assert_eq!(
            snapshot_hash(&snapshot),
            11_365_372_762_637_202_352,
            "\n{}",
            snapshot.character_grid()
        );
        assert!(
            display
                .snapshot()
                .character_grid()
                .contains("TRANSMISSION PAUSED")
        );
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::Playing);

        app.update(SimulationStep {
            tick: SimulationTick(2),
        });
        assert_eq!(app.state_kind(), AppStateKind::GameOver);
        assert!(
            app.semantic_tree()
                .root
                .children
                .iter()
                .any(|node| node.id.as_str() == "game-over.continue")
        );
        app.render(&mut display).expect("game-over screen renders");
        let snapshot = display.snapshot();
        assert_eq!(
            snapshot_hash(&snapshot),
            12_537_496_149_893_182_665,
            "\n{}",
            snapshot.character_grid()
        );
        assert!(
            display
                .snapshot()
                .character_grid()
                .contains("SIGNAL CAPACITY EXCEEDED")
        );
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::TagEntry);
        assert!(
            app.semantic_tree()
                .root
                .children
                .iter()
                .any(|node| node.id.as_str() == "tag.submit")
        );
        app.render(&mut display).expect("tag screen renders");
        let snapshot = display.snapshot();
        assert_eq!(
            snapshot_hash(&snapshot),
            14_200_273_076_630_531_459,
            "\n{}",
            snapshot.character_grid()
        );
        assert!(
            display
                .snapshot()
                .character_grid()
                .contains("ENTER OPERATOR IDENTIFICATION")
        );
        press(&mut app, AppAction::TextInput('D'));
        press(&mut app, AppAction::TextInput('R'));
        press(&mut app, AppAction::TextInput('X'));
        press(&mut app, AppAction::Confirm);
        assert_eq!(app.state_kind(), AppStateKind::Scores);

        app.render(&mut display).expect("score screen renders");
        let grid = display.snapshot().character_grid();
        assert!(grid.contains("DRX"));
        assert!(grid.contains("LOCAL RECORD SAVED"));
    }

    #[test]
    fn focus_loss_pauses_and_focus_return_does_not_resume() {
        let mut app = serviced_app();
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        app.update(SimulationStep {
            tick: SimulationTick(1),
        });

        app.handle_focus_lost();
        assert_eq!(app.state_kind(), AppStateKind::Paused);
        app.update(SimulationStep {
            tick: SimulationTick(2),
        });
        assert_eq!(app.state_kind(), AppStateKind::Paused);
    }

    #[test]
    fn focus_loss_while_resize_suspended_still_requires_game_resume() {
        let mut app = serviced_app();
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        app.update(SimulationStep {
            tick: SimulationTick(1),
        });

        app.handle_resize(80, 24);
        app.handle_focus_lost();
        app.handle_resize(100, 36);
        press(&mut app, AppAction::Confirm);

        assert_eq!(app.state_kind(), AppStateKind::Paused);
    }

    #[test]
    fn discovery_marker_is_persisted_when_a_matching_run_finishes() {
        let mut app = serviced_app();
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        press(&mut app, AppAction::Confirm);
        app.update(SimulationStep {
            tick: SimulationTick(1),
        });
        let request = match &app.state {
            AppState::Playing(session) => session.request.clone(),
            state => panic!("expected playing state, found {state:?}"),
        };

        app.finish_run(GameResult {
            game_id: request.game_id,
            mode_id: request.mode_id,
            rules_revision: request.rules_revision,
            seed: request.seed,
            final_tick: SimulationTick(90),
            score: 9_000,
            outcome: GameOutcome::GameOver,
            final_state_hash: StateHash(90),
            discoveries: vec![crate::DiscoveryMarker::PacketSweepTrace],
        });

        assert!(
            app.runtime
                .as_ref()
                .expect("runtime")
                .system_state
                .packet_sweep_trace_revealed
        );
        let mut display = raster_display::DisplayBuffer::canonical();
        app.render(&mut display).expect("game-over render");
        assert!(display.snapshot().character_grid().contains("TRACE90"));
    }

    #[derive(Debug)]
    struct TestRegistry;

    impl GameRegistry for TestRegistry {
        fn advertised_descriptors(&self) -> Vec<GameDescriptor> {
            vec![test_descriptor()]
        }

        fn hidden_descriptors(&self) -> Vec<GameDescriptor> {
            Vec::new()
        }

        fn create(&self, game_id: &GameId) -> Result<Box<dyn Game>, GameError> {
            if game_id.as_str() == "signal-stack" {
                Ok(Box::new(TestGame::new()))
            } else {
                Err(GameError::NotRegistered(game_id.clone()))
            }
        }
    }

    #[derive(Debug)]
    struct TestGame {
        descriptor: GameDescriptor,
        request: Option<NewRunRequest>,
        finished: bool,
        paused: bool,
    }

    impl TestGame {
        fn new() -> Self {
            Self {
                descriptor: test_descriptor(),
                request: None,
                finished: false,
                paused: false,
            }
        }
    }

    impl Game for TestGame {
        fn descriptor(&self) -> &GameDescriptor {
            &self.descriptor
        }

        fn reset(&mut self, request: &NewRunRequest) -> Result<(), GameError> {
            self.request = Some(request.clone());
            self.finished = false;
            Ok(())
        }

        fn handle_action(
            &mut self,
            _action: GameAction,
            _phase: ActionPhase,
        ) -> Result<(), GameError> {
            Ok(())
        }

        fn update(&mut self, _step: SimulationStep) -> Result<(), GameError> {
            if !self.paused {
                self.finished = true;
            }
            Ok(())
        }

        fn render(&self, _display: &mut dyn Display) -> Result<(), GlyphError> {
            Ok(())
        }

        fn set_paused(&mut self, paused: bool) {
            self.paused = paused;
        }

        fn status(&self) -> GameStatus {
            if self.finished {
                GameStatus::Finished
            } else {
                GameStatus::Running
            }
        }

        fn result(&self) -> Option<GameResult> {
            let request = self.request.as_ref()?;
            self.finished.then(|| GameResult {
                game_id: request.game_id.clone(),
                mode_id: request.mode_id.clone(),
                rules_revision: request.rules_revision,
                seed: request.seed,
                final_tick: SimulationTick(2),
                score: 100,
                outcome: GameOutcome::GameOver,
                final_state_hash: StateHash(42),
                discoveries: Vec::new(),
            })
        }
    }

    fn test_descriptor() -> GameDescriptor {
        GameDescriptor {
            id: GameId::parse("signal-stack").expect("valid ID"),
            title: "Signal Stack".to_owned(),
            short_title: "Signal Stack".to_owned(),
            category: crate::GameCategory::Puzzle,
            fictional_release_date: Some("21.11.1995".to_owned()),
            fictional_version: "1.4".to_owned(),
            catalog_number: Some("TEST-001".to_owned()),
            fictional_developer: "Frankenberg Logic Bureau".to_owned(),
            fictional_publisher: "Sara Circuitworks".to_owned(),
            premise: "Test".to_owned(),
            visibility: crate::CatalogVisibility::Advertised,
            rules_revision: RulesRevision::new(1).expect("valid revision"),
            minimum_grid: DISPLAY_SIZE,
            modes: vec![ModeDescriptor {
                id: ModeId::parse("standard-transmission").expect("valid ID"),
                title: "Standard Transmission".to_owned(),
            }],
            controls: vec![crate::ControlDescription {
                action: GameAction::Primary,
                label: "Test".to_owned(),
                default_bindings: vec!["X".to_owned()],
            }],
        }
    }

    #[derive(Debug, Default)]
    struct TestRepository {
        settings: Settings,
        scores: Vec<ScoreRecord>,
        system: SystemState,
    }

    impl SettingsRepository for TestRepository {
        fn load_settings(&mut self) -> Result<Settings, PersistenceError> {
            Ok(self.settings.clone())
        }

        fn save_settings(&mut self, settings: &Settings) -> Result<(), PersistenceError> {
            self.settings = settings.clone();
            Ok(())
        }
    }

    impl ScoreRepository for TestRepository {
        fn load_scores(&mut self) -> Result<Vec<ScoreRecord>, PersistenceError> {
            Ok(self.scores.clone())
        }

        fn save_scores(&mut self, scores: &[ScoreRecord]) -> Result<(), PersistenceError> {
            self.scores = scores.to_vec();
            Ok(())
        }
    }

    impl SystemStateRepository for TestRepository {
        fn load_system_state(&mut self) -> Result<SystemState, PersistenceError> {
            Ok(self.system.clone())
        }

        fn save_system_state(&mut self, state: &SystemState) -> Result<(), PersistenceError> {
            self.system = state.clone();
            Ok(())
        }
    }

    #[derive(Debug)]
    struct SharedScoreRepository {
        scores: Arc<Mutex<Vec<ScoreRecord>>>,
    }

    impl SettingsRepository for SharedScoreRepository {
        fn load_settings(&mut self) -> Result<Settings, PersistenceError> {
            Ok(Settings::default())
        }

        fn save_settings(&mut self, _settings: &Settings) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    impl ScoreRepository for SharedScoreRepository {
        fn load_scores(&mut self) -> Result<Vec<ScoreRecord>, PersistenceError> {
            Ok(self.scores.lock().expect("score lock").clone())
        }

        fn save_scores(&mut self, scores: &[ScoreRecord]) -> Result<(), PersistenceError> {
            *self.scores.lock().expect("score lock") = scores.to_vec();
            Ok(())
        }
    }

    impl SystemStateRepository for SharedScoreRepository {
        fn load_system_state(&mut self) -> Result<SystemState, PersistenceError> {
            Ok(SystemState::default())
        }

        fn save_system_state(&mut self, _state: &SystemState) -> Result<(), PersistenceError> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct TestMetadata;

    impl RunMetadataSource for TestMetadata {
        fn next_seed(&mut self) -> RunSeed {
            RunSeed(7)
        }

        fn unix_seconds(&self) -> i64 {
            1_753_481_600
        }
    }

    fn snapshot_hash(snapshot: &raster_display::DisplaySnapshot) -> u64 {
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
