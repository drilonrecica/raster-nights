# Raster Nights Technical Architecture

**Status:** Accepted working architecture  
**Audience:** Coding agents, maintainers, reviewers, and technical readers  
**Scope:** Shared engine, native terminal host, browser host, persistence, content, testing, and release boundaries

This document defines how Raster Nights should be structured. It focuses on stable boundaries, dependency direction, deterministic behavior, platform integration, and testability. It intentionally avoids pinning exact dependency versions unless a version is required by the repository itself.

---

## 1. Architecture goals

The architecture must support:

1. one authoritative implementation of each game’s rules;
2. native terminal and browser hosts from the beginning;
3. deterministic real-time simulation;
4. character-cell rendering through a canonical 100×36 display;
5. safe terminal lifecycle management;
6. local persistence with no backend dependency;
7. straightforward addition of owner-authored games;
8. automated testing without requiring a real terminal;
9. simple CI and release operations;
10. enough performance for a pseudo-3D racer and runner.

The architecture should be understandable to a single maintainer and coding agents. Abstraction is justified only when it protects a real boundary or enables testing.

---

## 2. Architecture non-goals

The architecture does not need to support:

- third-party plugins;
- runtime-loaded games;
- public binary compatibility;
- external scripting;
- a generic reusable terminal game engine as a separate product;
- multiplayer;
- cloud saves;
- server authority;
- dynamic content downloads;
- arbitrary game save states;
- a full operating-system emulator;
- a browser shell connected to a real PTY;
- a large async runtime;
- an Entity Component System by default.

If a future game genuinely needs a specialized internal pattern, it may use one inside its module without converting the entire project.

---

## 3. System context

```text
                    ┌──────────────────────────────┐
                    │       Raster Nights         │
                    │ shared engine and games      │
                    └──────────────┬───────────────┘
                                   │
                    normalized input / cell output
                                   │
                 ┌─────────────────┴─────────────────┐
                 │                                   │
        ┌────────▼────────┐                 ┌────────▼────────┐
        │ Native host      │                 │ Browser host     │
        │ Ratatui          │                 │ Rust + Wasm      │
        │ Crossterm        │                 │ Ratzilla         │
        │ local filesystem │                 │ browser storage  │
        └────────┬────────┘                 └────────┬────────┘
                 │                                   │
      local terminal / SSH / tmux           WebGL2 or Canvas
```

The hosts own platform concerns. The shared engine owns machine state, launcher behavior, game simulation, normalized input, and canonical cell composition.

---

## 4. Cargo workspace

Recommended structure:

```text
Cargo.toml
apps/
├── terminal/
│   ├── Cargo.toml
│   └── src/
└── web/
    ├── Cargo.toml
    └── src/
crates/
├── raster-engine/
├── raster-display/
├── raster-games/
├── raster-storage/
├── raster-audio/
└── raster-testkit/
```

### 4.1 `apps/terminal`

Owns:

- Crossterm event polling;
- raw mode;
- alternate screen;
- cursor visibility;
- native mouse capture;
- terminal resize events;
- native frame scheduling;
- panic-safe terminal cleanup;
- native storage adapter;
- terminal capability detection;
- native CLI parsing;
- process exit behavior.

It may depend on:

- `raster-engine`;
- `raster-display`;
- `raster-games`;
- `raster-storage`;
- `raster-audio` when introduced;
- Ratatui;
- Crossterm;
- native-only filesystem and directory helpers;
- CLI and logging libraries.

It must not contain game rules.

### 4.2 `apps/web`

Owns:

- `wasm-bindgen` exports;
- Ratzilla backend setup;
- WebGL2 selection and Canvas fallback;
- browser keyboard, mouse, touch, focus, and visibility events;
- `requestAnimationFrame`;
- browser storage adapter;
- browser audio adapter;
- fullscreen;
- host-to-site integration;
- error presentation.

It may depend on:

- `raster-engine`;
- `raster-display`;
- `raster-games`;
- `raster-storage`;
- `raster-audio` when introduced;
- Ratzilla;
- `wasm-bindgen`;
- selected `web-sys` APIs.

It must not contain game rules.

### 4.3 `raster-engine`

Owns:

- application state machine;
- machine lifecycle;
- boot and shutdown sequencing;
- launcher and software-detail state;
- normalized global actions;
- fixed-step clock concepts;
- run-seed and deterministic substream-derivation types;
- session state;
- game lifecycle traits;
- score/result envelope;
- shared errors and identifiers;
- storage repository port traits and domain records;
- semantic audio event and sink interfaces;
- host-independent semantic UI descriptions;
- injected game-registry interfaces;
- host-facing application API.

It must not depend on:

- `raster-games`;
- `raster-storage`;
- `raster-audio`;
- Crossterm;
- Ratzilla;
- `web-sys`;
- native filesystem APIs;
- browser storage;
- actual audio playback.

### 4.4 `raster-display`

Owns:

- canonical grid size;
- grid points, sizes, rectangles, and viewports;
- project display façade;
- cell, style, color, and modifier types or adapters;
- drawing primitives;
- text clipping;
- border primitives;
- viewport calculations;
- structured rendering snapshots;
- glyph validation;
- theme application.

It may use Ratatui core buffer and styling primitives behind project types.

Games should depend on the project façade, not higher-level Ratatui widgets.

### 4.5 `raster-games`

Owns all official game modules:

```text
src/
├── registry.rs
├── signal_stack/
├── bureau_9/
├── mnemonic_nullway/
├── afterline_99/
├── loopback/
├── hazard_registry/
├── relay_breaker/
└── packet_sweep/
```

It depends only on appropriate shared crates and deterministic utility dependencies.

It must not access platform APIs or storage directly.

### 4.6 `raster-storage`

Owns:

- persisted data-transfer and schema types;
- format versions;
- migrations;
- atomic-write helpers where platform-independent;
- corruption recovery policy;
- in-memory test adapter.

It implements repository ports defined by `raster-engine` and may depend on
engine domain types. Native and browser adapters live in their host crates when
platform APIs are required.

### 4.7 `raster-audio`

Owns:

- no-op and host-neutral audio-sink implementations;
- mapping semantic engine events to named assets or synthesis parameters;
- shared volume and mute behavior that is not platform I/O.

Semantic audio events and the sink port live in `raster-engine`, preventing an
engine/audio dependency cycle. Actual native or browser playback remains in host
adapters.

### 4.8 `raster-testkit`

Owns:

- deterministic test clocks;
- seeded contexts;
- input playback helpers;
- golden-run runner;
- display snapshot helpers;
- fixtures around the `raster-storage` in-memory adapter;
- fake audio sink;
- state hash assertions;
- common fixture builders.

---

## 5. Dependency direction

Allowed compile-time direction:

```text
raster-engine  -> raster-display
raster-games   -> raster-engine, raster-display
raster-storage -> raster-engine
raster-audio   -> raster-engine

apps/terminal  -> raster-engine, raster-display, raster-games,
                  raster-storage, optional raster-audio
apps/web       -> raster-engine, raster-display, raster-games,
                  raster-storage, optional raster-audio

raster-testkit -> shared crates needed by a test
```

Arrows point from a dependent to its dependency. Host applications are the
composition roots: they obtain registrations from `raster-games`, construct
storage/audio adapters, and inject those implementations into `raster-engine`.

Disallowed examples:

```text
raster-games -> crossterm
raster-games -> ratzilla
raster-games -> web-sys
raster-engine -> raster-games
raster-engine -> raster-storage
raster-engine -> raster-audio
raster-engine -> native filesystem
raster-display -> browser DOM
raster-storage -> game-specific simulation logic
```

Avoid circular dependencies. When two shared crates need each other, reconsider ownership of the shared type.

---

## 6. Application state machine

The top-level application should use explicit states.

Conceptual states:

```rust
enum AppState {
    PrivacyNotice(PrivacyNoticeState),
    ColdBoot(BootState),
    WarmBoot(BootState),
    Launcher(LauncherState),
    SoftwareDetails(SoftwareDetailsState),
    Loading(LoadingState),
    Playing(GameSession),
    Paused(PauseState),
    GameOver(GameOverState),
    TagEntry(TagEntryState),
    Scores(ScoresState),
    Shell(ShellState),
    Settings(SettingsState),
    Manual(ManualState),
    ResizeSuspended(ResizeSuspendedState),
    Shutdown(ShutdownState),
    FatalError(FatalErrorState),
}
```

Transitions should be explicit and testable.

Avoid a collection of flags such as:

```text
is_booting
is_paused
show_settings
is_in_game
needs_tag
```

because invalid combinations become possible.

The app state machine owns global presentation and delegates active gameplay to a game session.

---

## 7. Game lifecycle

A game is an internal curated component. The trait is not a public compatibility promise.

Conceptual interface:

```rust
pub trait Game {
    fn descriptor(&self) -> &'static GameDescriptor;

    fn reset(
        &mut self,
        request: NewRunRequest,
        services: &mut GameServices,
    ) -> Result<(), GameError>;

    fn handle_action(
        &mut self,
        action: GameAction,
        services: &mut GameServices,
    ) -> Result<(), GameError>;

    fn update(
        &mut self,
        step: SimulationStep,
        services: &mut GameServices,
    ) -> Result<(), GameError>;

    fn render(
        &self,
        display: &mut dyn Display,
        context: RenderContext,
    ) -> Result<(), RenderError>;

    fn status(&self) -> GameStatus;

    fn result(&self) -> Option<GameResult>;
}
```

The exact interface may evolve, but it must preserve:

- input normalization;
- fixed-step update;
- read-only rendering;
- explicit status;
- explicit result;
- injected services.

### 7.1 `GameDescriptor`

Conceptual metadata:

```rust
pub struct GameDescriptor {
    pub id: GameId,
    pub title: &'static str,
    pub short_title: &'static str,
    pub category: GameCategory,
    pub fictional_release_date: FictionalDate,
    pub fictional_version: &'static str,
    pub rules_revision: u16,
    pub minimum_grid: GridSize,
    pub supported_modes: &'static [ModeDescriptor],
    pub controls: &'static [ControlDescription],
}
```

Metadata must be sufficient for launcher, details, manuals, and result records.

### 7.2 Registry

Use an explicit owner-maintained registry supplied by the composition root.

```rust
pub static GAMES: &[GameRegistration] = &[
    signal_stack::REGISTRATION,
    bureau_9::REGISTRATION,
    mnemonic_nullway::REGISTRATION,
    afterline_99::REGISTRATION,
    loopback::REGISTRATION,
    hazard_registry::REGISTRATION,
    relay_breaker::REGISTRATION,
];
```

Hidden software is registered separately and omitted from ordinary catalog queries.

No linker tricks, runtime discovery, plugin manifests, or build-script scanning.
`raster-engine` operates on the injected registration slice and never imports
`raster-games`.

---

## 8. Normalized input architecture

Platform hosts produce raw events.

```text
Crossterm KeyEvent
Browser KeyboardEvent
Mouse event
Touch gesture
```

Host adapters translate them into normalized device events:

```rust
enum DeviceInput {
    KeyPressed(PhysicalKey),
    KeyRepeated(PhysicalKey),
    KeyReleased(PhysicalKey),
    PointerMoved(GridPoint),
    PointerPressed(PointerButton, GridPoint),
    PointerReleased(PointerButton, GridPoint),
    TouchGesture(TouchGesture),
    FocusLost,
    FocusGained,
    Resize(GridSize),
}
```

A shared input system maps device input into semantic actions:

```rust
enum AppAction {
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
```

### Held-key normalization

The shared input system maintains:

```rust
struct HeldInput {
    pressed_tick: SimulationTick,
    last_repeat_tick: SimulationTick,
    is_down: bool,
}
```

This avoids OS-specific repeat timing.

Hosts also report an input capability:

```rust
enum InputCapability {
    Enhanced,
    Compatibility,
}
```

Enhanced mode provides distinct press, repeat, and release events. On Unix
terminals, the native host requests Crossterm keyboard enhancement flags for
event types and all-key escape reporting only after confirming support.

Traditional terminal protocols report presses only. Compatibility mode:

1. delivers the first raw press immediately;
2. recognizes subsequent same-key presses as evidence that the key remains held;
3. uses those events only to refresh a short hold lease;
4. generates repeated semantic actions at engine-defined intervals;
5. releases the logical key when its lease expires.

Compatibility-mode constants are expressed in simulation ticks and covered by
tests. This mode cannot guarantee exact key-up timing, simultaneous held keys, or
the same analog precision as enhanced mode. These limitations are reported by
`display-test`, while recorded normalized actions remain deterministic.

### Text entry

When a text input context is active:

- printable characters are routed to the text editor;
- Vim navigation letters are not interpreted globally;
- command history and tag-entry rules are context-specific.

### Focus and resize

- Focus loss becomes a global pause/suspend transition.
- Resize below minimum becomes `ResizeSuspended`.
- These transitions freeze simulation before another update step.

---

## 9. Fixed-step simulation

### 9.1 Canonical tick rate

Initial authoritative simulation rate:

```rust
pub const SIMULATION_HZ: u32 = 60;
```

One step is conceptually 1/60 second.

### 9.2 Host frame loop

Hosts may render at different rates, but simulation advances in fixed steps.

Conceptual accumulator:

```rust
accumulator += elapsed;

while accumulator >= FIXED_STEP {
    app.update(FIXED_STEP)?;
    accumulator -= FIXED_STEP;
}

app.render(&mut display)?;
```

Clamp excessive accumulated time after long stalls. A backgrounded browser or suspended terminal should pause rather than simulate a large catch-up burst.

### 9.3 Turn-based games

Bureau 9 may render and update mostly on input events. It still participates in the same lifecycle and uses simulation ticks for timers where needed.

### 9.4 Time representation

Authoritative timing uses integer ticks.

Examples:

- countdown duration;
- lock delay;
- corruption duration;
- checkpoint time;
- combo windows;
- input repeat.

Wall-clock time is used only to measure host elapsed duration before conversion to simulation steps.

---

## 10. Deterministic randomization

Use a deterministic seeded generator suitable for identical native and Wasm output.

A new run receives:

```rust
pub struct RunSeed(pub u64);
```

Game code must not:

- call platform random APIs;
- seed from wall clock internally;
- create untracked secondary random generators;
- depend on hash-map iteration order for game outcomes.

Substreams may be derived deterministically:

```text
master run seed
├── gameplay sequence seed
├── cosmetic-only seed
└── authored-section seed
```

Cosmetic randomness must not influence authoritative state.

Signal Stack uses a shuffled seven-packet bag.

Mnemonic Nullway and Afterline 99 use deterministic section or route assembly where randomness applies.

---

## 11. Authoritative math

Use the simplest deterministic representation appropriate to each game.

### Integers

Use ordinary integers for:

- grids;
- scores;
- piece geometry;
- puzzle cells;
- counters;
- ticks;
- discrete lane indexes.

### Fixed-point

Use fixed-point or scaled integers for:

- steering;
- speed;
- acceleration;
- runner lateral interpolation;
- collision thresholds;
- boost heat;
- signal integrity calculations;
- pseudo-3D authoritative progress.

### Floating point

`f32` is permitted for renderer-only projection, interpolation, visual particles, and other non-authoritative presentation.

If a floating-point value influences scoring, collision, or route outcome, it must either:

- be replaced with deterministic fixed-point; or
- be covered by native/Wasm golden tests and documented as safe.

The preferred default is fixed-point for authoritative race and runner state.

---

## 12. State hashing and golden runs

### 12.1 Run record

Conceptual internal record:

```rust
pub struct RunRecord {
    pub format_version: u16,
    pub game_id: GameId,
    pub rules_revision: u16,
    pub mode_id: ModeId,
    pub seed: RunSeed,
    pub simulation_hz: u16,
    pub actions: Vec<TimedAction>,
    pub expected_final_hash: Option<StateHash>,
}
```

### 12.2 State hash

The authoritative state hash includes:

- simulation state;
- score;
- mode;
- seed-dependent sequence position;
- authoritative timers;
- collision state;
- game status.

It excludes:

- renderer interpolation;
- browser or terminal dimensions after canonical viewport resolution;
- audio;
- UI hover state;
- real timestamps;
- diagnostics;
- platform-specific event metadata.

### 12.3 Golden tests

Golden runs should cover:

- start and game-over;
- pause and resume;
- rapid input;
- edge collisions;
- high-speed behavior;
- deterministic random sequence;
- score boundaries;
- focus or resize normalization where relevant.

The same run data should execute in native Rust tests and Wasm tests.

---

## 13. Display architecture

### 13.1 Canonical grid

```rust
pub const DISPLAY_WIDTH: u16 = 100;
pub const DISPLAY_HEIGHT: u16 = 36;
```

### 13.2 Project display façade

Games should not receive a raw terminal backend.

Conceptual API:

```rust
pub trait Display {
    fn size(&self) -> GridSize;
    fn clear(&mut self, style: CellStyle);
    fn put(&mut self, point: GridPoint, cell: GameCell);
    fn text(&mut self, point: GridPoint, text: &str, style: TextStyle);
    fn fill_rect(&mut self, rect: GridRect, cell: GameCell);
    fn border(&mut self, rect: GridRect, border: BorderStyle);
    fn line(&mut self, from: GridPoint, to: GridPoint, cell: GameCell);
    fn clip(&mut self, rect: GridRect) -> DisplayViewport<'_>;
}
```

The implementation writes to a Ratatui-compatible in-memory buffer.

### 13.3 Cell model

A cell contains:

- one tested single-cell glyph or short grapheme known to occupy one cell;
- foreground color;
- background color;
- modifiers;
- optional semantic role for accessibility or snapshots.

Do not assume arbitrary Unicode text can be placed without width measurement.

### 13.4 Theme separation

Games request semantic styles:

```text
SystemText
SystemAccent
Warning
Critical
Muted
Player
Obstacle
Collectible
BoardGrid
Selected
```

Themes resolve semantic roles to colors and modifiers. Games may define game-specific semantic roles through controlled style tables.

### 13.5 Rendering flow

```text
App/game state
    ↓
project display façade
    ↓
canonical cell buffer
    ├── native Ratatui backend
    └── browser Ratzilla backend
```

Browser CRT post-processing occurs after cell composition.

---

## 14. Native terminal host

### 14.1 Initialization

Order:

1. inspect terminal size and capabilities;
2. create a restoration guard;
3. enter alternate screen;
4. enable raw mode;
5. enable supported keyboard enhancements;
6. hide cursor;
7. enable mouse capture where configured;
8. clear display;
9. start application loop.

If initialization fails, restore any already-applied state before returning an error.

### 14.2 Restoration guard

Use an RAII guard that attempts to restore:

- raw mode off;
- alternate screen exit;
- mouse capture off;
- keyboard enhancement flags popped;
- cursor visible;
- line wrapping and other modified input/display modes restored;
- terminal style reset.

Install a panic hook that delegates to cleanup before presenting the panic report where practical.

Do not use `process::exit` before cleanup unless the process is already in an unrecoverable state.

### 14.3 Event loop

Use a synchronous loop:

- Crossterm poll;
- drain available events;
- translate input;
- advance fixed-step accumulator;
- render;
- sleep/yield briefly if ahead of frame budget.

No Tokio in v1.

### 14.4 Differential rendering

Prefer backend-supported differential updates. Avoid clearing and repainting the entire physical terminal through ad hoc ANSI writes each frame.

### 14.5 Terminal capability profile

Automatic detection may consider:

- UTF-8 locale;
- `$TERM`;
- color capability;
- true-color hints;
- tmux;
- SSH;
- terminal size.

Allow overrides:

```bash
raster-nights --display auto
raster-nights --display 256
raster-nights --display truecolor
raster-nights --no-effects
```

Do not promise perfect capability detection.

`display-test` reports `INPUT MODE: ENHANCED` or
`INPUT MODE: COMPATIBILITY` and explains the compatibility limitations.

---

## 15. Browser host

### 15.1 Website boundary

The public website is a separate static site.

The WebAssembly application is mounted into an interactive display region after explicit power-on.

The website owns:

- page navigation;
- marketing content;
- installation documentation;
- surrounding accessibility semantics;
- `POWER ON`;
- fullscreen and mute controls;
- Wasm loading error UI;
- the browser semantic accessibility mirror.

Rust owns:

- DRX-90 machine UI;
- launcher;
- games;
- settings UI;
- input mapping;
- canonical display;
- local game state;
- host-independent semantic UI descriptions.

### 15.2 Build and loading

Build `apps/web` with `wasm-pack --target web --no-pack`. Generated JavaScript
and WebAssembly artifacts go to the ignored `website/public/wasm/` directory and
are served as static website assets.

The website dynamically imports and initializes the generated module only after
`POWER ON DRX-90`. Do not introduce Trunk as a second website build system.
Development and CI scripts own the exact commands.

### 15.3 Renderer selection

Preferred order:

1. WebGL2 Ratzilla backend;
2. Canvas Ratzilla backend;
3. clear unsupported-browser message.

The DOM backend is not the standard active-game renderer.

### 15.4 Animation loop

Use `requestAnimationFrame`.

On each callback:

1. obtain elapsed host time;
2. translate pending browser events;
3. pause if document is hidden or focus is lost;
4. advance fixed simulation steps;
5. render the canonical buffer;
6. submit to Ratzilla;
7. request next frame.

### 15.5 Input focus

Pressing `POWER ON` focuses the display.

The host shows:

- `INPUT LINK: ACTIVE`; or
- `INPUT LINK: CLICK DISPLAY TO RECONNECT`.

Browser-reserved shortcuts should not be aggressively blocked. Prevent default behavior only for keys actively used while the display is focused and where doing so does not interfere with critical browser controls.

### 15.6 Touch

Touch events are translated into semantic actions. Hardware-styled controls are website or host overlays whose actions enter the same normalized input system.

Real-time gameplay requires a landscape viewport. Portrait mode may render the
website and system screens but presents a rotate-device prompt before gameplay.
It never crops the canonical grid.

### 15.7 Semantic accessibility

The shared application exposes a read-only semantic tree for supported screens.
The browser host maps it to visually hidden native HTML controls and status
regions.

Minimum shared shape:

```rust
pub struct SemanticUiTree {
    pub revision: u64,
    pub root: SemanticNode,
}

pub struct SemanticNode {
    pub id: SemanticId,
    pub role: SemanticRole,
    pub label: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub state: SemanticState,
    pub actions: Vec<SemanticActionKind>,
    pub children: Vec<SemanticNode>,
}

pub struct SemanticEvent {
    pub id: SemanticId,
    pub command: SemanticCommand,
}
```

`SemanticRole` initially covers application, dialog, heading, list, list item,
button, status, text input, grid, row, and grid cell. `SemanticState` carries
focused, selected, disabled, expanded, and live-region flags where relevant.
Action kinds advertise activate, focus, increment, decrement, set text, and
supported grid movement. `SemanticCommand` carries any text or direction
payload. Stable `SemanticId` values allow the browser to update existing DOM
nodes instead of replacing the entire tree.

Requirements:

- semantic and cell focus identify the same logical element;
- DOM actions become normalized application actions;
- state and validation remain in Rust;
- updates are batched to avoid rebuilding unchanged nodes every frame;
- real-time games are not required to expose every gameplay cell;
- Bureau 9 exposes its board as an accessible grid.

### 15.8 Audio

Browser audio must initialize only after user interaction. Game code emits semantic events; the browser adapter owns actual playback.

---

## 16. Storage architecture

### 16.1 Storage trait

`raster-engine` defines the higher-level settings, score, puzzle-record, and
system-state repository ports. `raster-storage` defines and implements the
lower-level byte storage abstraction used by its codecs and adapters.

Conceptual interface:

```rust
pub trait Storage {
    fn read(&self, key: StorageKey) -> Result<Option<Vec<u8>>, StorageError>;
    fn write(&mut self, key: StorageKey, data: &[u8]) -> Result<(), StorageError>;
    fn remove(&mut self, key: StorageKey) -> Result<(), StorageError>;
    fn list(&self, namespace: StorageNamespace) -> Result<Vec<StorageKey>, StorageError>;
}
```

Engine repository ports expose typed operations:

```rust
trait SettingsRepository
trait ScoreRepository
trait PuzzleRecordRepository
trait SystemStateRepository
```

### 16.2 Native data

Recommended logical files:

```text
settings.toml
scores.json
bureau-9.json
system-state.json
diagnostics/
```

Use platform-appropriate per-user directories.

Writes should be atomic:

1. serialize;
2. write temporary sibling file;
3. flush where appropriate;
4. rename over destination.

### 16.3 Browser data

Use a browser-local adapter.

Start with the simplest robust option suitable for data size. If localStorage is used:

- keep payloads small;
- version schemas;
- handle quota errors;
- do not assume writes always succeed.

IndexedDB may be introduced when required by larger data or audio/content caching, but v1 content is bundled and records are small.

### 16.4 Recovery

On parse failure:

- preserve the original corrupt data where possible;
- record a local diagnostic;
- restore defaults for the affected domain;
- show a clear user message;
- do not erase unrelated records.

If storage initialization or a write fails, the application reports that
persistence is unavailable and continues with an in-memory repository. The
current session remains playable, but the UI must not imply that settings or
scores were saved.

### 16.5 No live save states

Do not serialize arbitrary game object graphs for mid-run recovery in v1.

Persist committed results and stable settings only.

---

## 17. Audio architecture

Games emit semantic cues:

```rust
enum AudioEvent {
    System(SystemAudioCue),
    Game {
        game_id: GameId,
        cue: GameAudioCue,
    },
    Music(MusicCommand),
}
```

Examples:

- MenuMove
- MenuConfirm
- BootBeep
- Countdown
- Collision
- Clear
- Warning
- GameOver
- MusicIntensity

The browser adapter maps cues to audio assets or synthesis.

The native default adapter is no-op.

Visual effects must communicate all critical audio information.

---

## 18. Localization architecture

v1 ships English only, but user-facing strings should be identified through stable keys.

Conceptual interface:

```rust
pub trait TextCatalog {
    fn text(&self, key: TextKey) -> Cow<'_, str>;
    fn format(&self, key: TextKey, args: &FormatArgs) -> String;
}
```

Requirements:

- game rules do not depend on displayed English strings;
- layouts tolerate moderately longer text;
- titles and company names remain canonical proper nouns;
- keyboard shortcuts are represented separately from prose;
- date presentation remains `DD.MM.YYYY` for fictional materials;
- canonical glyph limitations are documented before promising complex-script support.

Do not build a translation-management platform before translations exist.

---

## 19. Authored content architecture

Data-driven content may include:

- game tuning;
- vehicle specifications;
- route definitions;
- runner sections;
- challenge definitions;
- puzzle catalog;
- manuals;
- filesystem lore;
- studio profiles;
- reviews;
- audio mappings;
- palette definitions.

### Validation

Official content must be validated through tests or a command such as:

```bash
raster-nights validate-content
```

Validation should check:

- required IDs;
- duplicate IDs;
- references to missing studios or games;
- fictional date range;
- layout length limits;
- puzzle uniqueness metadata;
- route compatibility;
- value ranges;
- canonical names;
- asset existence.

Invalid official content should fail development checks.

---

## 20. Game implementation notes

## 20.1 Signal Stack

Authoritative state includes:

- matrix cells;
- active packet;
- packet position and rotation;
- hold packet;
- hold availability;
- preview queue;
- bag state;
- score;
- cleared channels;
- transmission rate;
- lock timer;
- combo state;
- sustained-transmission state;
- status.

Rendering is derived from state.

Packet geometry and rotation tests must be exhaustive.

## 20.2 Bureau 9

Authoritative state includes:

- case ID;
- initial givens;
- current entries;
- pencil marks;
- assistance profile;
- undo/redo history;
- elapsed ticks;
- hints;
- mistakes;
- completion status.

Puzzle validation and hint explanations should be separated from UI rendering.

## 20.3 Mnemonic Nullway

Use authored section definitions with validated entry and exit contracts.

Authoritative state includes:

- REC-0 longitudinal progress;
- lateral position;
- vertical state;
- phase;
- corruption;
- resources;
- active sections;
- collectible state;
- score;
- domain;
- deterministic generator position.

Projection is presentation-only.

## 20.4 Afterline 99

Separate:

- authoritative track progress and craft physics;
- route topology;
- rival simulation;
- collision;
- scoring;
- renderer projection.

Authoritative position should use track-relative fixed-point coordinates, not renderer row positions.

The V-SCAPE module may be shared with Mnemonic Nullway only for projection and validated section geometry. Do not force shared gameplay abstractions.

---

## 21. Error handling

### Categories

- recoverable user input errors;
- recoverable storage errors;
- unsupported display errors;
- content validation errors;
- host initialization errors;
- internal invariant failures.

### User-facing errors

Errors should be:

- clear;
- actionable;
- styled in R/OS voice where safe;
- accompanied by ordinary technical details when necessary.

Do not hide serious failures behind jokes.

Example:

```text
DISPLAY INITIALIZATION FAILED

Required: 100 × 36
Current: 84 × 27

Resize the terminal to continue.
Press Q to exit.
```

### Diagnostics

Structured tracing may record:

- state transitions;
- storage failures;
- render backend selection;
- content validation failures;
- simulation seed;
- rules revision;
- panic details.

It must not record sensitive environment data by default.

---

## 22. Testing architecture

### 22.1 Unit tests

Cover:

- game rules;
- scoring;
- rotation;
- collision;
- puzzle techniques;
- route branching;
- input mapping;
- state transitions;
- migrations.

### 22.2 Golden runs

Run identical input sequences and compare:

- final state hash;
- score;
- status;
- deterministic sequence state.

### 22.3 Rendering snapshots

Snapshot:

- 100×36 character grid;
- style map;
- structured cells.

Use image snapshots only for website and browser post-processing.

### 22.4 Host tests

Native:

- event mapping;
- resize suspension;
- cleanup guard;
- CLI parsing;
- storage path behavior.

Web:

- Wasm build;
- headless Wasm execution of shared golden runs;
- focus pause;
- browser input mapping;
- WebGL2-to-Canvas fallback logic;
- browser persistence smoke test.
- semantic-tree focus, action, and update behavior.

### 22.5 Content tests

Validate:

- puzzle catalog;
- dates;
- IDs;
- route and section data;
- references;
- layout-safe text;
- asset manifests.

---

## 23. CI architecture

CI must remain simple.

Recommended checks on pushes to `master`:

```text
format
clippy
workspace tests
workspace build
wasm build
headless Wasm golden tests
website build
```

A lightweight macOS compile check may run on `master` or release tags. Fedora/Linux is the primary CI execution environment.

Avoid:

- huge browser matrices;
- code signing;
- notarization;
- elaborate promotion workflows;
- automatically publishing every commit;
- long multi-stage release pipelines.

Tagged release workflow:

1. build supported native archives;
2. build web bundle;
3. generate SHA-256 checksums;
4. publish GitHub release;
5. deploy static website;
6. update Homebrew tap.

---

## 24. Performance architecture

Measure before optimizing.

Potential hot areas:

- browser full-grid updates;
- pseudo-3D projection;
- rival and obstacle iteration;
- text shaping;
- cell diff generation;
- audio event bursts.

Preferred strategies:

- reuse buffers;
- avoid allocation in per-tick hot loops;
- use compact deterministic state;
- prevalidate authored sections;
- precompute immutable geometry;
- separate update and rendering;
- batch browser rendering through backend;
- use semantic dirty regions only if profiling justifies complexity.

Do not sacrifice architectural clarity for speculative optimization.

---

## 25. Security and privacy architecture

The system executes only compiled official code and bundled content.

Do not introduce:

- arbitrary file execution;
- shell command execution;
- real filesystem browsing through the fictional shell;
- PTY attachment;
- remote URLs from content files;
- dynamic code loading;
- unsafe deserialization of untrusted binary blobs;
- automatic diagnostics submission.

The fictional filesystem is a curated virtual filesystem, not a view of the user’s disk.

Command shell commands operate only on the virtual system and official application actions.

---

## 26. Architecture evolution rules

A significant architecture change requires:

1. a concrete problem;
2. alternatives considered;
3. impact on native and web;
4. impact on determinism;
5. impact on release simplicity;
6. a decision entry in `DECISIONS.md`;
7. updates to this document;
8. migration plan where persisted data is affected.

Examples requiring explicit decisions:

- introducing Tokio;
- adding a backend service;
- changing canonical grid size;
- making games separate crates;
- adding plugin support;
- using floating point authoritatively;
- replacing Ratzilla;
- changing storage format;
- adding Windows to required support;
- introducing code signing.

---

## 27. Initial implementation sequence

Milestone 0 should proceed in this order:

1. workspace and shared identifiers;
2. display façade and 100×36 buffer;
3. deterministic simulation clock;
4. normalized input;
5. native restoration guard and renderer;
6. browser renderer and animation loop;
7. top-level app state machine;
8. privacy notice and boot;
9. launcher and game registry;
10. Signal Stack core simulation;
11. Signal Stack rendering;
12. pause, game over, tag entry;
13. local settings and score repository;
14. browser semantic mirror for implemented system screens;
15. website integration and Wasm loading.

See `docs/plans/001-first-signal.md` for detailed tasks and acceptance criteria.

After Milestone 0, version 0.1 adds Loopback, hidden Packet Sweep, the complete
website content, release archives, and Homebrew flow.

---

## 28. Architectural definition of done

The architecture is serving the product correctly when:

- one game implementation behaves identically across hosts;
- game modules have no platform dependencies;
- terminal restoration is reliable;
- browser rendering is smooth;
- deterministic golden runs match;
- storage failures do not destroy unrelated data;
- content is validated;
- adding an owner-authored game requires a module and explicit registry entry, not host rewrites;
- CI and release remain understandable;
- no hidden network behavior exists.
