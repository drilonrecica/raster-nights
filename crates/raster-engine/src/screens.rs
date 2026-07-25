// SPDX-License-Identifier: MPL-2.0

use raster_display::{
    BorderStyle, CellModifiers, CellStyle, Display, GlyphError, GridPoint, GridRect, SemanticColor,
};

use crate::{
    Application, CalendarDate, HostKind,
    app::{
        AppState, GameOverItem, GameOverState, PauseMenuItem, PauseReason, PauseState,
        SettingsMenuItem, SettingsState, SystemMenuItem, TagEntryState,
    },
};

const SCREEN: GridRect = GridRect::new(0, 0, 100, 36);

pub(crate) fn render(app: &Application, display: &mut dyn Display) -> Result<(), GlyphError> {
    display.clear(style(SemanticColor::Text));
    match app.state() {
        AppState::PrivacyNotice => render_privacy(display, app.host())?,
        AppState::PrivacyReview => render_privacy_review(display, app.host())?,
        AppState::ColdBoot(boot) => {
            render_boot(display, app.date(), boot.elapsed_ticks, false)?;
        }
        AppState::WarmBoot(boot) => {
            render_boot(display, app.date(), boot.elapsed_ticks, true)?;
        }
        AppState::Launcher => render_launcher(
            display,
            &app.advertised_descriptors(),
            app.selected_descriptor().as_ref(),
            app.persistence_warning(),
        )?,
        AppState::SoftwareDetails => render_details(
            display,
            app.selected_descriptor().as_ref(),
            app.best_selected_score(),
            app.packet_trace_available(),
        )?,
        AppState::ManualIndex(selected) => {
            render_manual_index(display, &app.manual_descriptors(), *selected)?
        }
        AppState::ManualDetail(id) => {
            render_manual_detail(display, app.manual_descriptor(id).as_ref())?
        }
        AppState::TraceEntry(trace) => render_trace_entry(display, &trace.value)?,
        AppState::Loading(request) => render_loading(
            display,
            app.selected_descriptor().as_ref(),
            &request.game_id,
        )?,
        AppState::Playing(session) => session.game.render(display)?,
        AppState::Paused(pause) => {
            pause.session.game.render(display)?;
            render_pause(display, pause)?;
        }
        AppState::Controls(_) => render_controls(display)?,
        AppState::Settings(settings) => {
            render_settings(display, settings, &app.settings(), app.host())?
        }
        AppState::GameOver(game_over) => render_game_over(display, game_over)?,
        AppState::TagEntry(tag) => render_tag_entry(display, tag)?,
        AppState::Scores(scores) => render_scores(
            display,
            &app.ranked_score_rows(&scores.key),
            scores.saved,
            app.persistence_warning(),
        )?,
        AppState::SystemMenu(menu) => render_system_menu(display, menu.selected)?,
        AppState::InterruptConfirm(_) => render_interrupt(display)?,
        AppState::ResizeSuspended(resize) => {
            render_resize(display, resize.columns, resize.rows, resize.ready_to_resume)?
        }
        AppState::Shutdown(shutdown) => render_shutdown(display, shutdown.elapsed_ticks)?,
        AppState::FatalError(message) => render_fatal(display, message)?,
        AppState::Transitioning => {
            unreachable!("transition sentinel is never externally observable")
        }
    }
    Ok(())
}

fn render_privacy(display: &mut dyn Display, host: HostKind) -> Result<(), GlyphError> {
    render_privacy_copy(display, host)?;
    emphasized(display, 37, 24, "[ ENTER ] CONTINUE")?;
    status_line(display, "ENTER Continue")?;
    Ok(())
}

fn render_privacy_review(display: &mut dyn Display, host: HostKind) -> Result<(), GlyphError> {
    render_privacy_copy(display, host)?;
    emphasized(display, 31, 24, "[ ENTER ] RETURN TO AFTERHOURS")?;
    status_line(display, "ENTER or ESC Return")?;
    Ok(())
}

fn render_privacy_copy(display: &mut dyn Display, host: HostKind) -> Result<(), GlyphError> {
    frame(display, "LOCAL SYSTEM NOTICE")?;
    panel(
        display,
        GridRect::new(12, 8, 76, 20),
        SemanticColor::Primary,
    )?;
    text(
        display,
        17,
        11,
        "Raster Nights has no accounts, analytics, telemetry or",
    )?;
    text(
        display,
        17,
        13,
        "advertising. It sends no scores or gameplay data.",
    )?;
    text(
        display,
        17,
        16,
        "Settings and high scores remain on this device.",
    )?;
    match host {
        HostKind::Native => {
            text(
                display,
                17,
                19,
                "The installed application makes no outbound requests.",
            )?;
        }
        HostKind::Browser => {
            text(
                display,
                17,
                19,
                "Browser play downloads only bundled site and application files.",
            )?;
        }
    }
    Ok(())
}

fn render_boot(
    display: &mut dyn Display,
    date: CalendarDate,
    elapsed_ticks: u64,
    warm: bool,
) -> Result<(), GlyphError> {
    frame(
        display,
        if warm {
            "R/OS WARM START"
        } else {
            "DRX-90 SYSTEM START"
        },
    )?;
    emphasized(display, 5, 3, "RECICA COMPUTER WORKS")?;
    text(display, 5, 5, "DRX-90 PERSONAL MULTIMEDIA SYSTEM")?;
    text(display, 5, 7, "R/OS ROM BIOS 3.11")?;

    let cold_lines = [
        "CPU TEST ............................. OK",
        "BASE MEMORY ......................... 640K",
        "EXTENDED MEMORY ..................... 16384K",
        "VECTOR DISPLAY ADAPTER .............. READY",
        "V/A-16 AUDIO ARRAY .................. READY",
        "AFTERHOURS ENTERTAINMENT MODULE ..... FOUND",
        "UNACCOUNTED RESIDENT PROCESS ........ 1",
    ];
    let warm_lines = [
        "CORE SERVICES ........................ READY",
        "DISPLAY STATE ....................... RESTORED",
        "AFTERHOURS INDEX .................... READY",
    ];
    let lines: &[&str] = if warm { &warm_lines } else { &cold_lines };
    let visible = usize::try_from(elapsed_ticks / 24)
        .unwrap_or(usize::MAX)
        .saturating_add(1)
        .min(lines.len());
    for (index, line) in lines.iter().take(visible).enumerate() {
        text(display, 8, 11 + index as u16 * 2, line)?;
    }

    if date.is_outside_certified_period() && visible == lines.len() {
        warning(
            display,
            8,
            27,
            &format!(
                "SYSTEM CLOCK ........ OUTSIDE WARRANTY PERIOD  {:02}.{:02}.{:04}",
                date.day, date.month, date.year
            ),
        )?;
    }
    status_line(display, "ANY KEY Skip diagnostics")?;
    Ok(())
}

fn render_launcher(
    display: &mut dyn Display,
    descriptors: &[crate::GameDescriptor],
    current: Option<&crate::GameDescriptor>,
    persistence_warning: Option<&str>,
) -> Result<(), GlyphError> {
    frame(display, "AFTERHOURS SOFTWARE ARCHIVE")?;
    muted(
        display,
        3,
        2,
        "[F3] LOCAL RECORDS                              [F10] R/OS",
    )?;
    horizontal_rule(display, 3)?;
    emphasized(display, 4, 6, "FEATURED SOFTWARE")?;

    if descriptors.is_empty() {
        selected(
            display,
            4,
            9,
            "> SIGNAL STACK             PUZZLE       21.11.1995",
        )?;
        muted(
            display,
            6,
            11,
            "Falling-packet transmission alignment / STANDARD TRANSMISSION",
        )?;
    }
    for (index, descriptor) in descriptors.iter().enumerate() {
        let row = 9 + index as u16 * 4;
        let line = format!(
            "{} {:<24} {:<12} {}",
            if current.is_some_and(|item| item.id == descriptor.id) {
                ">"
            } else {
                " "
            },
            descriptor.title.to_uppercase(),
            format!("{:?}", descriptor.category).to_uppercase(),
            descriptor
                .fictional_release_date
                .as_deref()
                .unwrap_or("UNLISTED")
        );
        if current.is_some_and(|item| item.id == descriptor.id) {
            selected(display, 4, row, &line)?;
        } else {
            text(display, 4, row, &line)?;
        }
        muted(display, 6, row + 2, &descriptor.premise)?;
    }
    text(
        display,
        4,
        20,
        &format!("OFFICIAL ARCHIVE INDEX: {} PROGRAMS", descriptors.len()),
    )?;
    if persistence_warning.is_some() {
        warning(
            display,
            4,
            29,
            "LOCAL STORAGE DEGRADED - RECORDS MAY NOT SURVIVE THIS SESSION",
        )?;
    }
    status_line(display, "UP/DOWN Select    ENTER Details    ESC System")?;
    Ok(())
}

fn render_details(
    display: &mut dyn Display,
    descriptor: Option<&crate::GameDescriptor>,
    best_score: Option<u64>,
    trace_available: bool,
) -> Result<(), GlyphError> {
    let Some(descriptor) = descriptor else {
        return render_fatal(display, "No software descriptor is available");
    };
    frame(display, "AFTERHOURS / SOFTWARE DETAILS")?;
    emphasized(display, 5, 4, &descriptor.title.to_uppercase())?;
    text(
        display,
        5,
        6,
        &format!(
            "{} / {} / {}",
            descriptor.fictional_developer,
            descriptor.fictional_publisher,
            descriptor
                .fictional_release_date
                .as_deref()
                .unwrap_or("UNLISTED")
        ),
    )?;
    muted(display, 5, 9, &descriptor.premise)?;
    let mode = descriptor
        .modes
        .first()
        .map_or("UNAVAILABLE", |mode| mode.title.as_str());
    text(
        display,
        5,
        13,
        &format!("MODE          {}", mode.to_uppercase()),
    )?;
    text(
        display,
        5,
        17,
        &best_score.map_or_else(
            || "LOCAL RECORD  NO RECORDS".to_owned(),
            |score| format!("LOCAL RECORD  {score:010}"),
        ),
    )?;
    for (index, control) in descriptor.controls.iter().take(4).enumerate() {
        text(
            display,
            5,
            21 + index as u16,
            &format!(
                "{:<14} {}",
                control.label.to_uppercase(),
                control.default_bindings.join(" / ")
            ),
        )?;
    }
    emphasized(
        display,
        5,
        28,
        &format!("[ ENTER ] START {}", mode.to_uppercase()),
    )?;
    if trace_available {
        warning(display, 58, 28, "[ T ] RESIDUAL TRACE")?;
        status_line(display, "ENTER Start    M Manual    T Trace    ESC Catalog")?;
    } else {
        status_line(display, "ENTER Start    M Manual    ESC Return to catalog")?;
    }
    Ok(())
}

fn render_loading(
    display: &mut dyn Display,
    descriptor: Option<&crate::GameDescriptor>,
    game_id: &crate::GameId,
) -> Result<(), GlyphError> {
    frame(display, "AFTERHOURS PROGRAM LOADER")?;
    let title = descriptor.filter(|item| item.id == *game_id).map_or_else(
        || game_id.to_string(),
        |item| format!("{} VERSION {}", item.title, item.fictional_version),
    );
    emphasized(display, 35, 13, &title.to_uppercase())?;
    text(display, 31, 17, "VERIFYING PROGRAM TABLES ............. OK")?;
    text(display, 31, 19, "ALLOCATING DISPLAY GRID .............. OK")?;
    muted(display, 34, 24, "PLEASE KEEP THE ARCHIVE DOOR CLOSED")?;
    status_line(display, "Loading local program core")?;
    Ok(())
}

fn render_manual_index(
    display: &mut dyn Display,
    manuals: &[crate::ManualDescriptor],
    selected_index: usize,
) -> Result<(), GlyphError> {
    frame(display, "R/OS MANUAL INDEX")?;
    emphasized(display, 5, 4, "INSTALLED SOFTWARE REFERENCES")?;
    for (index, manual) in manuals.iter().enumerate() {
        menu_item(
            display,
            7,
            9 + index as u16 * 5,
            &manual.title.to_uppercase(),
            index == selected_index,
        )?;
        muted(display, 9, 11 + index as u16 * 5, &manual.subtitle)?;
    }
    status_line(display, "UP/DOWN Select    ENTER Open    ESC Catalog")?;
    Ok(())
}

fn render_manual_detail(
    display: &mut dyn Display,
    manual: Option<&crate::ManualDescriptor>,
) -> Result<(), GlyphError> {
    let Some(manual) = manual else {
        return render_fatal(display, "Requested manual is unavailable");
    };
    frame(display, "R/OS SOFTWARE MANUAL")?;
    emphasized(display, 4, 3, &manual.title.to_uppercase())?;
    muted(display, 4, 5, &manual.subtitle)?;
    let mut row = 8;
    for section in manual.sections.iter().take(4) {
        emphasized(display, 4, row, &section.heading.to_uppercase())?;
        row += 2;
        if let Some(paragraph) = section.paragraphs.first() {
            for line in wrap_text(paragraph, 88, 2) {
                text(display, 6, row, &line)?;
                row += 1;
            }
        }
        row += 1;
    }
    status_line(display, "ESC Return to manual index")?;
    Ok(())
}

fn render_trace_entry(display: &mut dyn Display, value: &str) -> Result<(), GlyphError> {
    frame(display, "SIGNAL STACK / RESIDUAL TRACE")?;
    panel(
        display,
        GridRect::new(20, 9, 60, 17),
        SemanticColor::Warning,
    )?;
    warning(display, 29, 12, "NONSTANDARD TRACE CHANNEL DETECTED")?;
    text(display, 29, 16, "ENTER DIAGNOSTIC TRACE CODE")?;
    selected(display, 29, 19, &format!("> {value:<7}"))?;
    muted(display, 29, 23, "ENTER Submit    ESC Clear / Return")?;
    status_line(display, "Trace entry is local to this system")?;
    Ok(())
}

fn render_pause(display: &mut dyn Display, pause: &PauseState) -> Result<(), GlyphError> {
    panel(
        display,
        GridRect::new(27, 5, 46, 27),
        SemanticColor::Warning,
    )?;
    warning(
        display,
        40,
        7,
        match pause.reason {
            PauseReason::Player => "TRANSMISSION PAUSED",
            PauseReason::FocusLost => "FOCUS LOST - PAUSED",
        },
    )?;
    let items = [
        (PauseMenuItem::Resume, "RESUME"),
        (PauseMenuItem::Restart, "RESTART"),
        (PauseMenuItem::Controls, "CONTROLS"),
        (PauseMenuItem::Settings, "SETTINGS"),
        (PauseMenuItem::Return, "RETURN TO AFTERHOURS"),
        (PauseMenuItem::Shutdown, "SHUT DOWN"),
    ];
    for (index, (item, label)) in items.iter().enumerate() {
        menu_item(
            display,
            36,
            11 + index as u16 * 3,
            label,
            pause.selected == *item,
        )?;
    }
    Ok(())
}

fn render_controls(display: &mut dyn Display) -> Result<(), GlyphError> {
    frame(display, "SIGNAL STACK / CONTROLS")?;
    panel(
        display,
        GridRect::new(16, 5, 68, 25),
        SemanticColor::Primary,
    )?;
    text(display, 23, 9, "LEFT / RIGHT       MOVE PACKET")?;
    text(display, 23, 12, "UP OR X            ROTATE CLOCKWISE")?;
    text(
        display,
        23,
        15,
        "Z                  ROTATE COUNTERCLOCKWISE",
    )?;
    text(display, 23, 18, "DOWN               SOFT DROP")?;
    text(display, 23, 21, "SPACE              HARD DROP")?;
    text(display, 23, 24, "C                  HOLD PACKET")?;
    status_line(display, "ESC Return to pause menu")?;
    Ok(())
}

fn render_settings(
    display: &mut dyn Display,
    menu: &SettingsState,
    settings: &crate::Settings,
    host: HostKind,
) -> Result<(), GlyphError> {
    frame(display, "R/OS / SETTINGS")?;
    panel(
        display,
        GridRect::new(18, 6, 64, 24),
        SemanticColor::Primary,
    )?;
    emphasized(display, 25, 9, "LOCAL DISPLAY AND OPERATION")?;
    let rows = [
        (
            SettingsMenuItem::Palette,
            format!("DISPLAY PALETTE       {:?}", settings.display_palette).to_uppercase(),
        ),
        (
            SettingsMenuItem::ReducedMotion,
            format!(
                "REDUCED MOTION        {}",
                if settings.reduced_motion { "ON" } else { "OFF" }
            ),
        ),
        (
            SettingsMenuItem::QuietOperation,
            format!(
                "QUIET OPERATION       {}",
                if settings.quiet_operation {
                    "ON"
                } else {
                    "OFF"
                }
            ),
        ),
        (
            SettingsMenuItem::BrowserCrt,
            format!(
                "BROWSER CRT EFFECTS   {}{}",
                if settings.crt_effects { "ON" } else { "OFF" },
                if host == HostKind::Native {
                    " (WEB ONLY)"
                } else {
                    ""
                }
            ),
        ),
    ];
    for (index, (item, label)) in rows.iter().enumerate() {
        menu_item(
            display,
            25,
            13 + index as u16 * 3,
            label,
            menu.selected == *item,
        )?;
    }
    muted(
        display,
        25,
        27,
        "UP/DOWN Select   ENTER Change   ESC Return",
    )?;
    status_line(display, "Settings are local to this device")?;
    Ok(())
}

fn render_game_over(
    display: &mut dyn Display,
    game_over: &GameOverState,
) -> Result<(), GlyphError> {
    frame(display, "SIGNAL STACK / DIAGNOSTIC")?;
    warning(display, 35, 6, "SIGNAL CAPACITY EXCEEDED")?;
    text(
        display,
        26,
        11,
        "CHANNEL MATRIX ................. SATURATED",
    )?;
    text(display, 26, 13, "PACKET INGRESS ................. FAILED")?;
    text(
        display,
        26,
        15,
        "TRANSMISSION ................... TERMINATED",
    )?;
    if game_over
        .result
        .discoveries
        .contains(&crate::DiscoveryMarker::PacketSweepTrace)
    {
        warning(display, 35, 18, "RESIDUAL TRACE ............... TRACE90")?;
    }
    emphasized(
        display,
        39,
        20,
        &format!("SCORE {:010}", game_over.result.score),
    )?;
    if game_over.qualifies {
        emphasized(display, 37, 22, "LOCAL RECORD QUALIFIED")?;
    }
    menu_item(
        display,
        19,
        27,
        if game_over.qualifies {
            "ENTER TAG"
        } else {
            "VIEW SCORES"
        },
        game_over.selected == GameOverItem::Continue,
    )?;
    menu_item(
        display,
        43,
        27,
        "RESTART",
        game_over.selected == GameOverItem::Restart,
    )?;
    menu_item(
        display,
        62,
        27,
        "AFTERHOURS",
        game_over.selected == GameOverItem::Return,
    )?;
    status_line(display, "LEFT/RIGHT Select   ENTER Confirm")?;
    Ok(())
}

fn render_tag_entry(display: &mut dyn Display, tag: &TagEntryState) -> Result<(), GlyphError> {
    frame(display, "NEW LOCAL SYSTEM RECORD")?;
    panel(
        display,
        GridRect::new(18, 7, 64, 22),
        SemanticColor::Primary,
    )?;
    emphasized(display, 34, 11, &format!("SCORE {:010}", tag.result.score))?;
    text(display, 31, 16, "ENTER OPERATOR IDENTIFICATION")?;
    for index in 0..3 {
        let value = char::from(tag.tag[index]).to_string();
        if tag.cursor == index {
            selected(display, 44 + index as u16 * 4, 21, &value)?;
        } else {
            emphasized(display, 44 + index as u16 * 4, 21, &value)?;
        }
    }
    status_line(
        display,
        "TYPE or ARROWS Edit   ENTER Submit   ESC Diagnostics",
    )?;
    Ok(())
}

fn render_scores(
    display: &mut dyn Display,
    scores: &[&crate::ScoreRecord],
    saved: bool,
    warning_message: Option<&str>,
) -> Result<(), GlyphError> {
    frame(display, "AFTERHOURS / LOCAL RECORDS")?;
    emphasized(display, 5, 4, "SIGNAL STACK - STANDARD TRANSMISSION")?;
    muted(display, 5, 6, "RANK  TAG        SCORE     CHANNEL TIME")?;
    for (index, record) in scores.iter().enumerate() {
        text(
            display,
            5,
            9 + index as u16 * 2,
            &format!(
                "{:>2}.   {:3}   {:010}   {:>7} TICKS",
                index + 1,
                record.tag,
                record.score,
                record.duration.0
            ),
        )?;
    }
    if scores.is_empty() {
        muted(display, 5, 10, "NO LOCAL RECORDS")?;
    }
    if warning_message.is_some() {
        warning(
            display,
            5,
            30,
            "RECORD IS IN MEMORY ONLY - LOCAL SAVE FAILED",
        )?;
    } else if saved {
        emphasized(display, 5, 30, "LOCAL RECORD SAVED")?;
    }
    status_line(display, "ENTER or ESC Return to AfterHours")?;
    Ok(())
}

fn render_system_menu(
    display: &mut dyn Display,
    selected_item: SystemMenuItem,
) -> Result<(), GlyphError> {
    frame(display, "R/OS SYSTEM CONTROL")?;
    panel(
        display,
        GridRect::new(27, 9, 46, 18),
        SemanticColor::Primary,
    )?;
    emphasized(display, 39, 12, "SYSTEM CONTROL")?;
    menu_item(
        display,
        36,
        15,
        "RETURN TO AFTERHOURS",
        selected_item == SystemMenuItem::Return,
    )?;
    menu_item(
        display,
        36,
        17,
        "LOCAL SYSTEM NOTICE",
        selected_item == SystemMenuItem::Privacy,
    )?;
    menu_item(
        display,
        36,
        19,
        "SHUT DOWN",
        selected_item == SystemMenuItem::Shutdown,
    )?;
    muted(display, 34, 23, "UP/DOWN Select   ENTER Confirm")?;
    status_line(display, "ESC Return")?;
    Ok(())
}

fn render_interrupt(display: &mut dyn Display) -> Result<(), GlyphError> {
    frame(display, "R/OS INTERRUPT")?;
    panel(
        display,
        GridRect::new(23, 10, 54, 15),
        SemanticColor::Warning,
    )?;
    warning(display, 36, 13, "SAFE EXIT REQUESTED")?;
    text(
        display,
        29,
        16,
        "Shut down Raster Nights and restore the terminal?",
    )?;
    emphasized(display, 34, 20, "[ ENTER ] SHUT DOWN")?;
    status_line(display, "ESC Cancel    CTRL+C Exit immediately")?;
    Ok(())
}

fn render_resize(
    display: &mut dyn Display,
    columns: u16,
    rows: u16,
    ready: bool,
) -> Result<(), GlyphError> {
    frame(display, "SESSION SUSPENDED")?;
    panel(
        display,
        GridRect::new(20, 10, 60, 16),
        SemanticColor::Warning,
    )?;
    warning(display, 32, 13, "TERMINAL RESIZE REQUIRED")?;
    text(
        display,
        29,
        16,
        &format!("REQUIRED 100 X 36    CURRENT {columns} X {rows}"),
    )?;
    if ready {
        emphasized(display, 33, 21, "[ ENTER ] RESUME SESSION")?;
    } else {
        muted(
            display,
            28,
            21,
            "Restore the minimum dimensions to continue.",
        )?;
    }
    status_line(display, "Simulation and timers are suspended")?;
    Ok(())
}

fn render_shutdown(display: &mut dyn Display, elapsed_ticks: u64) -> Result<(), GlyphError> {
    frame(display, "R/OS SHUTDOWN")?;
    emphasized(display, 34, 12, "CLOSING AFTERHOURS ARCHIVE")?;
    let lines = [
        "FLUSHING LOCAL SYSTEM STATE .......... OK",
        "RELEASING DISPLAY ADAPTER ............ OK",
        "SYSTEM MAY NOW BE SWITCHED OFF.",
    ];
    let visible = usize::try_from(elapsed_ticks / 24)
        .unwrap_or(usize::MAX)
        .saturating_add(1)
        .min(lines.len());
    for (index, line) in lines.iter().take(visible).enumerate() {
        text(display, 29, 17 + index as u16 * 2, line)?;
    }
    status_line(display, "ANY KEY Exit now")?;
    Ok(())
}

fn render_fatal(display: &mut dyn Display, message: &str) -> Result<(), GlyphError> {
    frame(display, "R/OS SYSTEM FAULT")?;
    panel(display, GridRect::new(14, 9, 72, 18), SemanticColor::Danger)?;
    warning(display, 38, 12, "APPLICATION FAULT")?;
    text(display, 18, 16, &safe_line(message, 64))?;
    muted(
        display,
        18,
        20,
        "No diagnostic data was uploaded. Terminal restoration remains active.",
    )?;
    emphasized(display, 33, 24, "[ ENTER ] SHUT DOWN")?;
    status_line(display, "ENTER Safe shutdown")?;
    Ok(())
}

fn frame(display: &mut dyn Display, title: &str) -> Result<(), GlyphError> {
    display.border(
        SCREEN,
        BorderStyle::ascii(CellStyle::new(
            SemanticColor::Primary,
            SemanticColor::Background,
        )),
    )?;
    emphasized(display, 3, 0, &format!(" {title} "))?;
    Ok(())
}

fn panel(
    display: &mut dyn Display,
    rect: GridRect,
    color: SemanticColor,
) -> Result<(), GlyphError> {
    display.border(
        rect,
        BorderStyle::ascii(CellStyle::new(color, SemanticColor::Background)),
    )
}

fn horizontal_rule(display: &mut dyn Display, row: u16) -> Result<(), GlyphError> {
    display.text(
        GridPoint::new(1, row),
        &"-".repeat(98),
        style(SemanticColor::Primary),
    )
}

fn status_line(display: &mut dyn Display, value: &str) -> Result<(), GlyphError> {
    horizontal_rule(display, 33)?;
    muted(display, 3, 34, value)
}

fn menu_item(
    display: &mut dyn Display,
    x: u16,
    y: u16,
    value: &str,
    active: bool,
) -> Result<(), GlyphError> {
    if active {
        selected(display, x, y, &format!("> {value}"))
    } else {
        text(display, x + 2, y, value)
    }
}

fn text(display: &mut dyn Display, x: u16, y: u16, value: &str) -> Result<(), GlyphError> {
    display.text(GridPoint::new(x, y), value, style(SemanticColor::Text))
}

fn wrap_text(value: &str, width: usize, maximum_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut current));
            if lines.len() == maximum_lines {
                return lines;
            }
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() && lines.len() < maximum_lines {
        lines.push(current);
    }
    lines
}

fn emphasized(display: &mut dyn Display, x: u16, y: u16, value: &str) -> Result<(), GlyphError> {
    display.text(
        GridPoint::new(x, y),
        value,
        style(SemanticColor::Primary).bold(),
    )
}

fn muted(display: &mut dyn Display, x: u16, y: u16, value: &str) -> Result<(), GlyphError> {
    display.text(GridPoint::new(x, y), value, style(SemanticColor::Muted))
}

fn warning(display: &mut dyn Display, x: u16, y: u16, value: &str) -> Result<(), GlyphError> {
    display.text(GridPoint::new(x, y), value, style(SemanticColor::Warning))
}

fn selected(display: &mut dyn Display, x: u16, y: u16, value: &str) -> Result<(), GlyphError> {
    display.text(
        GridPoint::new(x, y),
        value,
        CellStyle {
            foreground: SemanticColor::Background,
            background: SemanticColor::Primary,
            modifiers: CellModifiers {
                bold: true,
                reversed: true,
                ..CellModifiers::default()
            },
        },
    )
}

const fn style(foreground: SemanticColor) -> CellStyle {
    CellStyle::new(foreground, SemanticColor::Background)
}

fn safe_line(value: &str, maximum: usize) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_graphic() || character == ' ' {
                character
            } else {
                '?'
            }
        })
        .take(maximum)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActionPhase, AppAction, HostKind};
    use raster_display::DisplayBuffer;

    fn rendered(app: &Application) -> String {
        let mut display = DisplayBuffer::canonical();
        app.render(&mut display).expect("screen should render");
        display.snapshot().character_grid()
    }

    #[test]
    fn privacy_snapshot_contains_unambiguous_native_network_copy() {
        let app = Application::new(HostKind::Native, CalendarDate::new(25, 7, 2026), false);
        let snapshot = rendered(&app);

        assert_eq!(snapshot.lines().count(), 36);
        assert!(snapshot.contains("makes no outbound requests"));
        assert!(snapshot.contains("[ ENTER ] CONTINUE"));
    }

    #[test]
    fn launcher_snapshot_contains_only_available_catalog_entry() {
        let mut app = Application::new(HostKind::Native, CalendarDate::new(25, 7, 2026), true);
        app.handle_action(AppAction::Confirm, ActionPhase::Pressed);
        let snapshot = rendered(&app);

        assert!(snapshot.contains("SIGNAL STACK"));
        assert!(!snapshot.contains("BUREAU 9"));
        assert!(!snapshot.contains("AFTERLINE 99"));
    }

    #[test]
    fn fatal_error_copy_is_sanitized_before_cell_rendering() {
        let mut app = Application::new(HostKind::Browser, CalendarDate::new(25, 7, 2026), true);
        app.fail("bad\n界");

        assert!(rendered(&app).contains("bad??"));
    }
}
