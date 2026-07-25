# Raster Nights Product Specification

**Status:** Working product specification  
**Audience:** Product owner, coding agents, designers, testers, and maintainers  
**Public product:** Raster Nights  
**Creator and curator:** Drilon Reçica

This document defines what Raster Nights is, who it serves, what experiences it must provide, what belongs in each release, and which behaviors are non-negotiable. It is the primary authority for user-facing product behavior.

Technical implementation details belong in `ARCHITECTURE.md`. Visual implementation details belong in `DESIGN.md`. Fictional names, histories, and chronology belong in `CANON.md`.

---

## 1. Product summary

Raster Nights is a curated collection of substantial retro games for a fictional 1990s home computer. The games run in two forms:

1. as a native terminal application, including through SSH and tmux;
2. as a browser application compiled from the same Rust game logic to WebAssembly.

The product presents the user with an original machine called the **DRX-90**, manufactured by **Reçica Computer Works**, running **R/OS**, and offering games through the **AfterHours** entertainment archive.

The project is designed to be valuable in three ways:

- as a playable arcade with genuinely good games;
- as a showcase of clean cross-platform Rust architecture;
- as a public-source, owner-curated creative software project.

The product is not a plugin platform, emulator of a real computer, generic terminal UI framework, multiplayer service, or community game repository.

---

## 2. Product promise

The central promise is:

> Raster Nights delivers polished, mechanically meaningful retro games for a computer that never existed, with the same core game behavior available natively in a terminal and in the browser.

The experience should produce several reactions:

- “This feels like a coherent lost computer platform.”
- “The games are more ambitious than ordinary terminal demos.”
- “It is fun to launch during a short wait in tmux or over SSH.”
- “The browser version feels deliberately retro rather than cheaply nostalgic.”
- “The architecture is unusually disciplined for a showcase project.”

---

## 3. Target users

### Primary users

Developers and technically inclined players who:

- enjoy terminals, SSH, tmux, and home-lab environments;
- appreciate Rust and WebAssembly;
- enjoy retro computer aesthetics;
- want short or medium arcade sessions between other work;
- are curious about unusual engineering constraints.

### Secondary users

- retro game enthusiasts;
- puzzle and arcade players;
- visitors discovering the project through the website;
- recruiters or engineers evaluating the creator’s product and technical judgment;
- educators or learners studying deterministic game architecture;
- users who enjoy fictional computer systems and software archaeology.

The experience must remain understandable to ordinary players. It may reward technical familiarity, but it must not require knowledge of Rust, terminal internals, or old operating systems.

---

## 4. Product principles

### 4.1 Games must justify the platform

The fact that a game runs in a terminal is not sufficient. Each flagship must provide:

- responsive controls;
- coherent rules;
- meaningful replay value;
- polished game-over and restart behavior;
- readable scoring;
- thoughtful difficulty;
- distinctive presentation;
- a reason to play again.

### 4.2 Terminal cells are a creative medium

The game world is built from character cells, block symbols, borders, typography, and color. Browser post-processing may enrich the presentation but cannot replace the canonical terminal-cell composition.

### 4.3 Native and browser are peers

Neither host is an afterthought. The same official game must have equivalent:

- rules;
- scoring;
- difficulty;
- simulation timing;
- random-seed behavior;
- game-over conditions;
- authoritative collision behavior.

Hosts may differ in audio, touch input, browser-only post-processing, local storage adapters, and fullscreen behavior.

### 4.4 The product is curated

Only the owner adds official games and canon. There is no:

- plugin SDK;
- game scripting layer;
- downloadable game package;
- dynamic library loading;
- public compatibility promise for third-party game modules;
- automated discovery of external games.

### 4.5 Local operation is a feature

Version 1 requires no backend service. The native and browser applications work with local content and local records.

No accounts, analytics, telemetry, update checks, cloud saves, advertisements, or automatic network requests.

### 4.6 Fiction is coherent but subordinate to playability

The fictional operating system, companies, release dates, manuals, and cross-game references should deepen the experience. They must not:

- obstruct fast access to games;
- force long unskippable sequences;
- confuse ordinary navigation;
- introduce privacy concerns;
- substitute lore for gameplay quality.

### 4.7 Accessibility is part of correctness

Accessibility features are not optional polish. The product must account for:

- color vision differences;
- flashing and motion sensitivity;
- keyboard-only users;
- mouse users;
- touch users where appropriate;
- audio-disabled environments;
- terminal font variation;
- browser focus changes;
- SSH latency and terminal resize events.

---

## 5. Product identity hierarchy

### Public identity

- **Project name:** Raster Nights
- **Repository name:** `raster-nights`
- **Canonical executable:** `raster-nights`
- **Optional short alias:** `rnights`
- **Primary tagline:** “Lost games for a computer that never existed.”
- **Technical description:** “A retro game system built in Rust for your terminal and the web.”

### Fictional identity

- **Manufacturer:** Reçica Computer Works
- **Computer:** DRX-90
- **Operating system:** R/OS
- **Arcade environment:** AfterHours
- **Resident utility/entity:** NUL — Nonessential Utility Layer
- **Fictional commercial period:** 05.10.1993–31.12.1999

The public project name and fictional software names must remain separate. Raster Nights is the real product. AfterHours is software inside the fictional machine.

---

## 6. Supported product surfaces

### 6.1 Native terminal application

The native application must support:

- local terminal launch;
- use inside tmux and GNU Screen;
- use through ordinary SSH sessions after installation on the remote machine;
- full keyboard operation;
- native mouse input in menus and compatible games;
- safe raw-mode and alternate-screen restoration;
- suspend-and-resume behavior during terminal resize;
- direct game launch from the command line.

The native application does not host a public SSH service. Users install it on their own systems.

### 6.2 Browser application

The browser product consists of:

- a readable project website;
- a prominent `POWER ON DRX-90` action;
- a WebAssembly game application;
- keyboard and mouse support;
- hardware-styled touch controls where appropriate;
- local browser persistence;
- optional audio;
- adjustable visual effects;
- no automatic startup or audio on page load.

### 6.3 Website

The website is not entirely trapped inside the fictional machine. It provides conventional access to:

- project summary;
- browser play;
- terminal installation;
- game catalog;
- screenshots or recordings;
- manuals and fictional documents;
- architecture overview;
- privacy statement;
- source repository;
- credits and licensing.

The visual direction resembles a polished 1990s product catalog or computer advertisement, interpreted through modern accessible web layout.

---

## 7. Canonical display and layout requirements

- Logical display size: **100 columns × 36 rows**
- Native minimum terminal size: **100×36**
- Browser scales the complete logical grid to fit the available area.
- Browser gameplay must not crop the grid to increase text size.
- When native dimensions are too small:
  - simulation pauses;
  - timers freeze;
  - current game state is retained in memory;
  - a resize requirement is displayed;
  - the user explicitly resumes after valid dimensions return.
- Canonical gameplay must avoid ambiguous-width Unicode and emoji-style glyphs.

Games may reserve internal regions for a board, road, HUD, or panels, but the outer logical display remains fixed.

---

## 8. Core user journey

### 8.1 Website first visit

1. User opens the Raster Nights website.
2. The hero explains terminal, SSH/tmux, and browser play.
3. User presses `POWER ON DRX-90`.
4. The display receives keyboard focus.
5. Optional sound remains off unless the user enabled it.
6. The first-run privacy notice appears.
7. A cold boot and POST sequence begins.
8. Any input may skip the remaining boot animation.
9. AfterHours opens to Featured Software.
10. User selects a game, sees its detail screen, and starts it.

### 8.2 Native first launch

1. User runs `raster-nights`.
2. Terminal capability and size are checked.
3. Terminal mode is initialized safely.
4. First-run privacy notice appears.
5. Cold boot begins.
6. AfterHours opens to Featured Software.
7. User navigates with keyboard or supported mouse input.
8. On exit, a short shutdown sequence runs unless the exit is urgent.
9. Terminal state is restored before process termination.

### 8.3 Returning session

- Default startup is a warm boot lasting approximately 2–3 seconds.
- Any input skips it.
- The launcher remembers:
  - last selected game;
  - catalog section;
  - last game mode;
  - last entered score tag;
  - preferred display and accessibility settings.
- A cold boot remains available through a reboot command or setting.

### 8.4 Quick launch

The native CLI supports direct launch:

```bash
raster-nights play signal-stack
raster-nights play loopback --quick
```

Normal direct launch shows a shortened publisher and controls flow.

`--quick` bypasses ceremonial boot, publisher, and loading sequences. Essential safety or controls information may remain unless explicitly skipped by another option.

---

## 9. Machine boot and operating environment

### 9.1 Cold boot

Target duration: 4–6 seconds before skipping.

The sequence should:

- begin plausibly;
- show CPU, memory, display, and storage initialization;
- introduce fictional devices;
- include occasional dry diagnostics;
- hint at one unexplained resident process;
- avoid becoming a joke list.

Cold boot appears:

- on first launch;
- after major version changes where appropriate;
- after explicit reboot;
- when the user selects cold boot.

### 9.2 Warm boot

Target duration: 2–3 seconds before skipping.

Warm boot:

- quickly verifies essential systems;
- restores the last launcher state;
- preserves the product identity without wasting time.

The boot cannot be permanently removed through normal settings. Quiet mode and direct launch provide practical shortcuts.

### 9.3 Launcher

The default launcher is a keyboard-driven graphical software archive.

Primary catalog categories:

- Featured Software
- Shareware and Cover-Disk Archive
- All Software
- By Genre
- By Studio
- By Release Year
- Recently Played
- System Control
- Command Prompt

The launcher is not a modern card grid. It should resemble period software catalog or operating-system UI.

### 9.4 Software detail screen

Pressing `Enter` on a game opens a detail screen containing:

- title;
- fictional developer and publisher;
- release date;
- one-sentence premise;
- mode and difficulty;
- local record;
- concise controls;
- `START`, `MANUAL`, and `RETURN`.

A second `Enter` starts the game.

### 9.5 Miniature command shell

R/OS provides a small shell with:

- current directory;
- command history;
- tab completion;
- quoted arguments;
- a curated fictional filesystem;
- executable aliases for games.

Supported commands may include:

```text
HELP
DIR
CD
TYPE
CLS
DATE
TIME
VER
MEM
PLAY
GAMES
SCORES
SET
REBOOT
SHUTDOWN
EXIT
NUL
```

The shell is not a scripting environment or real operating-system emulator.

### 9.6 Shutdown

Normal shutdown lasts approximately 1–2 seconds and is skippable.

Urgent exits, panics, terminal failure, or repeated `Ctrl+C` prioritize terminal restoration over theatrical presentation.

---

## 10. Session behavior

### Pause

Pause is always available during normal gameplay.

- `Esc` pauses gameplay.
- Browser focus loss pauses immediately.
- Hidden browser tabs pause immediately.
- Terminal resize below minimum pauses immediately.
- Returning focus does not resume automatically.
- Pause freezes authoritative timers and simulation.
- A pause menu offers Resume, Restart, Controls, Settings, Return to AfterHours, and Shut Down.

### `Ctrl+C`

- First `Ctrl+C`: show a short interrupt prompt.
- Second `Ctrl+C`: immediate safe exit.
- Noninteractive CLI commands follow normal command-line termination behavior.

### SSH interruption

- Previously written scores and settings remain safe.
- Mid-game process recovery is not provided.
- tmux or Screen is the recommended way to preserve a live session across disconnects.

### Crash behavior

- Restore terminal state where reasonably possible.
- Save a local sanitized diagnostic report where enabled.
- Never upload anything automatically.

---

## 11. Input requirements

### Global keyboard

- Arrow keys: primary navigation.
- `H`, `J`, `K`, `L`: Vim-style navigation outside text entry.
- `Enter`: confirm.
- `Esc`: pause or back according to context.
- `Ctrl+C`: safe interrupt.
- Text-entry modes temporarily disable navigation bindings for printable characters.

### Held input

The engine normalizes held-key behavior. It must not depend on operating-system repeat settings.

Actions track:

- pressed;
- held;
- released;
- held duration;
- engine-controlled repeat timing.

### Mouse

Native and browser mouse support is required for:

- launcher;
- settings;
- manuals;
- Bureau 9;
- compatible menus and lists.

Keyboard operation remains complete.

### Touch

Touch controls are styled as fictional DRX-90 hardware.

Full mobile support is expected for:

- Bureau 9;
- Hazard Registry;
- Signal Stack;
- Loopback;
- Relay Breaker.

Mnemonic Nullway and Afterline 99 receive best-effort landscape controls. Their desktop keyboard experience remains authoritative, and mobile must not indefinitely block v1.

### Remapping

Control remapping is part of v1 scope but may be cut after lower-priority visual/audio features if necessary.

Support:

- global action remapping;
- game-specific action remapping;
- conflict detection;
- restore defaults;
- one-handed presets where practical.

---

## 12. Settings

The primary settings UI exists inside R/OS.

Categories:

- Display
- Audio
- Input
- Accessibility
- Startup
- Storage
- Privacy
- Diagnostics

Website-level controls outside the emulated machine are limited to:

- Power
- Mute
- Fullscreen
- Reconnect Focus

### Display themes

Initial authored themes:

- RCW Standard
- Amber Office
- Green Phosphor
- Midnight VGA
- High Contrast
- Paper Terminal

Do not provide a full arbitrary color editor in v1.

### Effects

Profiles:

- Clean
- Authentic
- Intense
- Custom

Independent accessibility-sensitive controls:

- screen shake;
- flashing;
- flicker;
- motion intensity;
- rapid palette cycling.

Reduced Motion disables or minimizes all relevant effects.

### Quiet Operation

Quiet Operation reduces:

- boot duration;
- publisher cards;
- loading animations;
- shutdown ceremony;
- repeated help screens;
- nonessential NUL commentary.

It does not remove all fictional presentation.

### Factory reset

Two distinct operations:

1. **Reset System Settings**
   - restores configuration;
   - preserves scores and puzzle records.

2. **Erase Local Archive**
   - deletes settings, scores, tags, puzzle records, and remembered state;
   - requires explicit typed confirmation.

---

## 13. Persistence

Version 1 stores data locally per operating-system user or browser profile.

Expected logical data:

- settings;
- local scoreboards;
- Bureau 9 puzzle records;
- last launcher state;
- last score tag;
- privacy acknowledgement;
- optional diagnostics settings.

No named user accounts or multiple settings profiles.

Score records should include, where relevant:

- game ID;
- mode;
- score;
- duration;
- difficulty;
- seed or puzzle ID;
- completion status;
- rules revision;
- assistance profile;
- timestamp;
- three-character tag.

Stored data must be versioned and migrated or recovered safely.

---

## 14. Privacy requirements

Version 1 makes no automatic network requests.

The installed application must not:

- send analytics;
- send telemetry;
- check for updates;
- submit scores;
- fetch remote games;
- download manuals;
- load remote artwork;
- upload crashes;
- validate licenses online;
- contact a leaderboard.

First-run notice:

```text
LOCAL SYSTEM NOTICE

Raster Nights has no accounts, analytics, telemetry,
advertising, or automatic network activity.

Settings, puzzle records and high scores remain on this device.
```

Diagnostics:

- keep a small in-memory ring buffer;
- write a local report after crashes where possible;
- write on explicit diagnostic command;
- exclude usernames, home paths, hostnames, IPs, unrelated environment variables, typed tags, and arbitrary shell history;
- never transmit automatically.

---

## 15. Accessibility requirements

### Required in v1

- high-contrast palette;
- color-blind-safe palettes;
- reduced flashing;
- reduced motion;
- optional screen shake;
- optional CRT effects;
- visible alternatives for audio cues;
- complete keyboard launcher operation;
- native and browser mouse support where appropriate;
- remappable controls;
- one-handed presets where practical;
- no required distinction based solely on color;
- readable focus states;
- pausing on focus loss;
- assistance profiles clearly separated from presentational accessibility.

### Score eligibility

The following do not invalidate canonical scoring:

- color-blind themes;
- high contrast;
- disabled shake;
- reduced flashing;
- disabled CRT effects;
- remapped keys;
- larger browser scaling;
- visual replacement for audio.

The following produce assisted or noncanonical records:

- reduced simulation speed;
- extended timing windows;
- automatic steering;
- hazard reveals;
- direct solution checking;
- gameplay hints;
- invulnerability.

Accessibility must never be described as cheating. Records store a rules profile.

### Screen readers

Full screen-reader access to real-time cell games is not a v1 promise. The website, settings, menus, manuals, and Bureau 9 should use meaningful semantics where practical.

---

## 16. Audio

### Browser

Optional original audio:

- chiptune or tracker-style music;
- FM-inspired synthesis;
- short low-resolution samples;
- sound effects;
- reactive music layers.

Audio starts only after user interaction and remains off on first visit unless explicitly enabled.

### Native terminal

Silent by default.

Do not depend on terminal bell for gameplay. All meaningful sound information has a visible equivalent.

### Fictional hardware

Audio is attributed to the fictional **RCW V/A-16 Audio Array**, allowing period-spanning sound styles.

---

## 17. Version 0.1 scope

### Product name

**Raster Nights 0.1 — System Preview**

### Supported official targets

- macOS Apple Silicon
- macOS Intel
- Fedora-compatible Linux x86-64
- modern browsers with WebAssembly and WebGL2 or Canvas support

### Distribution

- Homebrew tap
- Cargo installation
- release archives
- browser website
- SHA-256 checksums
- no code signing or notarization

### Required product features

- public website;
- `POWER ON DRX-90`;
- cold and warm boot;
- first-run privacy notice;
- AfterHours launcher;
- software detail screens;
- basic System Control;
- local settings;
- local scoreboards;
- three-character score entry;
- keyboard and mouse;
- basic touch support;
- terminal resize suspension;
- browser focus suspension;
- safe terminal restoration;
- SSH and tmux documentation;
- deterministic shared simulation;
- native and web parity;
- substantial fictional content;
- no network requests.

### Included games

#### Signal Stack

Required for 0.1:

- Standard Transmission;
- 10×20 visible board plus hidden spawn rows;
- seven familiar four-cell packet geometries;
- five-piece preview;
- one hold per placed packet;
- shuffled seven-packet bag;
- wall kicks;
- soft and hard drop;
- phase-rotation scoring;
- signal chains;
- sustained transmission bonuses;
- zero-state matrix bonus;
- local high score;
- game-over diagnostic;
- score-tag entry;
- pause and restart;
- native and browser parity.

Allowed after 0.1:

- Burst Calibration;
- Transmission Repair;
- complete soundtrack;
- final attract mode;
- advanced challenge catalog.

#### Loopback

Required:

- Quick Circuit mode lasting approximately 90–180 seconds;
- immediate direct launch;
- network-port twist;
- local score;
- readable keyboard and touch controls;
- lightweight fictional presentation.

#### Packet Sweep

Required:

- hidden discovery condition;
- one arena;
- one-to-three-minute runs;
- one primary mechanic;
- local score;
- no catalog listing.

### Substantial fictional content

0.1 includes:

- DRX-90 product overview;
- Reçica Computer Works profile;
- R/OS help content;
- Signal Stack manual;
- Loopback cover-disk notes;
- selected studio profiles;
- two or three fictional reviews;
- curated filesystem lore;
- several NUL interactions;
- references to future software.

---

## 18. Version 1.0 scope

Version 1.0 requires:

- stable fictional identity;
- four complete flagship games;
- three advertised bonus games;
- one hidden game;
- native terminal and browser;
- local records;
- stable internal game lifecycle;
- robust deterministic tests;
- accessibility baseline;
- substantial website and manuals;
- simple official builds;
- macOS and Fedora support at minimum;
- no backend requirement.

### Flagship catalog

#### Signal Stack

Mechanically polished falling-block game with:

- Standard Transmission;
- Burst Calibration;
- Transmission Repair challenges.

#### Bureau 9

Curated standard 9×9 Sudoku:

- 240 reviewed puzzles;
- 60 per difficulty;
- Case Archive;
- Assign Case;
- Paper, Assisted, Guided;
- pencil marks;
- undo/redo;
- optional candidate removal;
- explanatory hints;
- local completion records.

#### Mnemonic Nullway

Finite 12–18-minute recovery runner:

- REC-0 player process;
- lane shifting, jump, slide, phase shift;
- visible and archived memory;
- corruption meter;
- memory fragments, checksums, clock cycles;
- hybrid authored procedural sections;
- multiple memory domains;
- finite ending;
- endless mode after completion.

#### Afterline 99

Primary showcase racer:

- six signal craft;
- five principal point-to-point routes;
- championship;
- single route;
- time attack;
- endless highway;
- branching paths;
- drafting;
- drifting;
- boost temperature;
- signal integrity;
- checkpoint deadlines;
- multiple ending outcomes.

### Bonus catalog

- Loopback
- Hazard Registry
- Relay Breaker

### Hidden catalog

- Packet Sweep

---

## 19. Game-specific requirements

## 19.1 Signal Stack

### Fictional metadata

- Release date: 21.11.1995
- Developer: Frankenberg Logic Bureau
- Publisher: Sara Circuitworks
- Technology: Vranidoll Signal Works
- Fictional version: 1.4

### Core rules

- 10×20 visible matrix
- 4 hidden spawn rows
- 5 previews
- 1 hold per placement
- shuffled seven-packet bag
- deterministic seed
- levels primarily advance every 10 cleared channels
- lock delay begins forgiving and tightens at high rates
- canonical mode ends on spawn failure or hidden-region lockout
- no emergency recovery in canonical scoring

### Modes

- Standard Transmission
- Burst Calibration
- Transmission Repair
- Packet Sweep, hidden

### Presentation

Industrial diagnostic chamber with saturated VGA packet colors. Critical board readability always overrides effects.

## 19.2 Bureau 9

### Fictional metadata

- Release date: 08.02.1994
- Developer: Hofheim Home Software
- Publisher: Prishtina Vector House
- Verification: Frankenberg Logic Bureau

### Puzzle catalog

- 60 Easy
- 60 Medium
- 60 Hard
- 60 Expert
- one verified solution
- known logical path
- no guessing required
- stable case ID

### Assistance

- Paper: no solution checking
- Assisted: duplicate and conflict assistance without direct solution comparison
- Guided: direct checking and explanatory hints

### Hints

Explain techniques, initially including:

- naked single;
- hidden single;
- locked candidate;
- naked pair;
- hidden pair.

### Active atmosphere

Quiet and distraction-free. NUL does not interrupt solving.

## 19.3 Mnemonic Nullway

### Fictional metadata

- Release date: 06.06.1997
- Developer: Nora Nova Interactive
- Publisher: Prishtina Vector House
- Technology: V-SCAPE Perspective System 2.2
- Additional technology: Noah Arc Labs

### Player

REC-0, an unauthorized recovery process.

### Core actions

- lane movement;
- jump;
- slide;
- phase shift.

### Failure

- accumulating corruption;
- severe hazards may terminate immediately;
- corruption affects presentation without destroying readability.

### Structure

- finite Recovery Session;
- Continuous Recovery endless mode;
- Sector Calibration challenges;
- deterministic authored-section assembly.

## 19.4 Afterline 99

### Fictional metadata

- Release date: 17.09.1998
- Developer: Nürnberg Night Systems
- Publisher: Reçica Computer Works
- Technology: V-SCAPE Perspective System 3.7
- Signal simulation: Vranidoll Signal Works
- Vehicle logic: Noah Arc Labs

### Core race model

- visible rivals;
- traffic;
- checkpoint deadlines;
- point-to-point championship;
- branching routes;
- simulated analog steering from held input;
- acceleration, brake, drift, drafting, boost;
- signal integrity rather than conventional health.

### Content target

- 6 craft
- 5 championship routes
- 1 endless highway
- 3 short challenge routes

### Ending outcomes

- Licensed Champion
- Night Champion
- Last Transmission
- Unclassified Result

---

## 20. Fictional release chronology

Initial public catalog chronology:

| Date | Title | Classification |
|---|---|---|
| 19.12.1993 | Loopback | Bonus |
| 08.02.1994 | Bureau 9 | Flagship |
| 27.05.1994 | Hazard Registry | Bonus |
| 21.11.1995 | Signal Stack | Flagship |
| 11.08.1996 | Relay Breaker | Bonus |
| 06.06.1997 | Mnemonic Nullway | Flagship |
| 17.09.1998 | Afterline 99 | Flagship |
| Unknown | Packet Sweep | Hidden |

Future real releases may fill any fictional date between 05.10.1993 and 31.12.1999. They do not have to move chronologically forward.

---

## 21. Local score behavior

### Tag entry

After a qualifying score:

- user enters exactly three characters;
- previous tag is preselected;
- submission remains explicit;
- accepted set includes A–Z, 0–9, and a restrained punctuation subset such as `-`, `.`, `_`.

### Canonical records

Records are separated when rule differences matter.

Examples:

- Signal Stack Standard Transmission
- Signal Stack Burst Calibration
- Bureau 9 per case and assistance profile
- Afterline 99 per route, mode, craft, and difficulty where necessary

### No global leaderboard in v1

A public top-ten leaderboard is explicitly deferred. Any future leaderboard is a separate optional service and must not make local play dependent on a backend.

---

## 22. Fictional content requirements

Every flagship game receives:

- title screen;
- developer;
- publisher;
- release date;
- fictional version;
- catalog number;
- copyright card;
- short manual;
- controls reference;
- software-detail screen;
- at least one cross-reference to the wider ecosystem.

Bonus games receive lighter shareware or cover-disk presentation.

Hidden games remain absent from ordinary catalog and marketing.

Fictional content must not expose real private details. Place and family-inspired names are transformed into brands without real biographies or relationship claims.

---

## 23. Product non-goals

The following are out of scope for v1 unless explicitly reconsidered:

- multiplayer;
- local two-player modes;
- accounts;
- global profiles;
- cloud saves;
- persistent campaign progression;
- unlock trees;
- virtual currency;
- game plugins;
- downloadable game code;
- public game SDK;
- backend services;
- hosted SSH service;
- analytics;
- telemetry;
- advertising;
- automatic update checks;
- dynamic remote content;
- authoritative online anti-cheat;
- replay export UI;
- screenshot export UI;
- arbitrary live-game save states;
- full screen-reader support for every real-time game;
- Windows as a required 0.1 platform;
- code signing and notarization;
- complex package-manager coverage.

---

## 24. Product quality gates

A tagged public release must satisfy:

### Functional

- native and browser launch;
- game loop completes;
- controls documented;
- pause, restart, exit work;
- local writes are safe;
- terminal restoration works;
- no known score corruption;
- no hidden network requests.

### Cross-platform

- shared rules pass deterministic tests;
- native and web produce matching golden-run hashes;
- 100×36 behavior is correct;
- browser focus handling works;
- terminal resize handling works.

### Visual

- no placeholder UI;
- text fits canonical grid;
- no ambiguous glyphs;
- critical state remains readable without color;
- effects obey accessibility settings.

### Fiction

- names and dates match canon;
- no invented real-life personal detail;
- humor matches tone;
- publisher and studio credits are consistent.

### Operational

- release archives build;
- website builds;
- Homebrew metadata can be updated;
- checksums are generated;
- release notes are written;
- CI remains understandable and short.

---

## 25. Scope-cut priority

When a milestone becomes too large, reduce in this order:

1. full soundtrack;
2. elaborate CRT effects;
3. advanced touch support for racer and runner;
4. deep shell functionality;
5. extensive filesystem lore;
6. polished remapping UI;
7. elaborate attract modes;
8. extra challenges;
9. secondary bonus-game polish.

Do not cut:

- native/browser parity;
- safe terminal behavior;
- deterministic simulation;
- core accessibility;
- Signal Stack canonical completeness for 0.1;
- coherent boot and launcher identity;
- local persistence;
- no-network privacy guarantee.

---

## 26. Definition of launch success

Success is not defined by a fixed GitHub-star target.

A successful public release means:

- strangers can install it without assistance;
- the browser version works;
- users play it through terminals, SSH, or tmux;
- Signal Stack is enjoyable beyond novelty;
- developers respect the architecture;
- the DRX-90 fiction feels cohesive;
- no backend or tracking is required;
- the project accurately represents the creator’s product and engineering judgment.

---

## 27. Post-1.0 direction

Priority order after 1.0:

1. additional substantial games;
2. stability and platform compatibility;
3. deeper cross-game lore;
4. improved art and audio;
5. stronger mobile experiences;
6. optional online experiments only when they provide clear value.

Popularity alone must not introduce accounts, plugins, telemetry, multiplayer, or permanent backend operations.

---

## 28. Open questions that remain intentionally open

These do not block early implementation:

- final public trademark and domain clearance for Raster Nights;
- final selected bundled web font;
- final music-production workflow;
- exact tuning values for game difficulty;
- future Windows support;
- whether a global recreational top-ten service is ever worth operating;
- final volume of fictional manuals after v1.

Agents must not silently decide these as permanent product changes. Use a plan or decision entry when they become relevant.
