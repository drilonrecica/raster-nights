# Raster Nights

> **Lost games for a computer that never existed.**

Raster Nights is a curated retro game system built in Rust. It runs as a native terminal application—including through ordinary SSH sessions and inside tmux—and as a browser application compiled to WebAssembly.

The public product presents an original fictional 1990s home computer:

- **Manufacturer:** Reçica Computer Works
- **Machine:** DRX-90
- **Operating system:** R/OS
- **Game archive:** AfterHours
- **Resident system entity:** NUL, the Nonessential Utility Layer

Raster Nights is simultaneously:

1. a polished playable arcade;
2. an engineering showcase for deterministic Rust game architecture across terminal and browser hosts; and
3. a public-source, owner-curated fictional software archive.

The official game catalog, fictional canon, art direction, names, music, and roadmap are curated by Drilon Reçica.

---

## Project status

Raster Nights is in the specification and initial implementation stage.

The first public milestone, **Raster Nights 0.1 — System Preview**, is planned to include:

- native terminal builds for macOS and Fedora-compatible Linux;
- a browser build;
- the DRX-90 cold and warm boot;
- the AfterHours launcher;
- local settings and high scores;
- **Signal Stack** as the first complete flagship game;
- **Loopback** as a fast one-to-three-minute bonus game;
- **Packet Sweep** as a hidden miniature game;
- keyboard, mouse, and basic touch input;
- SSH and tmux usability;
- no analytics, accounts, automatic network activity, or backend services.

See `docs/ROADMAP.md` for milestone sequencing.

---

## Why Raster Nights exists

Terminal games are usually small technical demonstrations, while browser retro experiences often imitate old hardware without actually sharing code with a native terminal application.

Raster Nights takes a stricter approach:

- the same Rust game rules run in both environments;
- games render into a canonical 100×36 terminal-cell display;
- native and browser hosts translate their own input and presentation into shared engine concepts;
- authoritative simulation is deterministic;
- the browser may add subtle CRT effects, but it does not replace the character-cell game renderer with ordinary sprites;
- native play remains practical over SSH and in tmux.

The result should be impressive because the games are genuinely good—not merely because they run in unusual places.

---

## Planned game archive

### Flagship software

#### Signal Stack — 21.11.1995

A falling-block transmission-alignment game developed by Frankenberg Logic Bureau and published by Sara Circuitworks.

Modes include:

- Standard Transmission;
- Burst Calibration;
- authored Transmission Repair scenarios.

#### Bureau 9 — 08.02.1994

A curated Sudoku archive presented as numerical compliance software.

It includes:

- 240 reviewed puzzles;
- Paper, Assisted, and Guided profiles;
- explanatory logical hints;
- keyboard, mouse, and touch input.

#### Mnemonic Nullway — 06.06.1997

A finite pseudo-3D recovery runner through damaged machine memory.

Its signature mechanics include:

- visible and archived memory layers;
- phase shifting;
- corruption management;
- authored procedural sections;
- multiple memory domains.

#### Afterline 99 — 17.09.1998

A pseudo-3D nocturnal signal racer and the primary showcase game.

It includes:

- six signal craft;
- five point-to-point championship routes;
- branching paths;
- drafting, drifting, boost heat, signal integrity, and checkpoint pressure.

### Bonus software

- **Loopback** — a rapid Snake-inspired network-routing game.
- **Hazard Registry** — a Minesweeper-inspired inspection utility.
- **Relay Breaker** — a Breakout-inspired signal-circuit game.

### Hidden software

- **Packet Sweep** — an undocumented miniature game concealed inside Signal Stack’s fiction.

The complete v1 target is four flagship games, three advertised bonus games, and one hidden game.

---

## Product principles

1. **Games first.**  
   A terminal aesthetic cannot excuse shallow mechanics, poor controls, or incomplete polish.

2. **One simulation, two hosts.**  
   Native and browser versions share rules, scoring, seeded randomness, and authoritative state.

3. **Character cells are the medium.**  
   The browser is allowed richer post-processing, not a separate sprite-based game implementation.

4. **Curated, not extensible.**  
   There is no plugin ecosystem or public game SDK. Games are compiled into the official application.

5. **Local by default and by design.**  
   No accounts, analytics, telemetry, automatic network requests, cloud saves, or hosted dependency in v1.

6. **Fiction with discipline.**  
   The fictional computer, studios, manuals, release dates, and operating-system personality form a coherent canon.

7. **Accessible presentation.**  
   Color-blind-safe themes, reduced effects, keyboard navigation, mouse support where appropriate, and visible alternatives to audio cues are baseline product requirements.

8. **Simple operations.**  
   Development happens directly on `master`. Releases use a straightforward tag-build-publish-deploy workflow.

---

## Technical direction

The intended architecture uses:

- Rust for all game logic and the machine interface;
- a project-owned display façade over Ratatui cell primitives;
- Ratatui and Crossterm for the native terminal host;
- Rust/WebAssembly and Ratzilla for the browser host;
- WebGL2 as the preferred browser renderer with Canvas fallback;
- Astro, HTML, CSS, and minimal TypeScript for the surrounding website;
- deterministic 60 Hz simulation;
- integer or fixed-point authoritative game state;
- TOML and JSON for local user-facing data;
- one Cargo workspace with separate native and browser applications.

See `docs/ARCHITECTURE.md` for the complete design.

---

## Canonical display

Raster Nights uses a fixed logical display:

```text
100 columns × 36 rows
```

Native terminals smaller than this suspend the session until resized.

The browser scales the complete grid to fit the available area. It may add subtle scanlines, glow, vignette, and other CRT-inspired presentation, but the gameplay coordinate system remains exactly 100×36.

---

## Privacy

Raster Nights is intentionally local.

The installed application does not:

- create accounts;
- track usage;
- send analytics;
- check for updates automatically;
- submit scores;
- fetch games or manuals;
- upload crashes;
- load remote content during normal operation.

Settings, high scores, puzzle records, and remembered system state remain on the local device.

Crash diagnostics, when available, are written locally and are never transmitted automatically.

---

## Installation targets

The initial official native targets are planned to include:

- macOS Apple Silicon;
- macOS Intel;
- Fedora-compatible Linux x86-64.

Planned installation options:

```bash
brew tap <owner>/raster-nights
brew install raster-nights
```

and:

```bash
cargo install raster-nights
```

Release archives and checksums will be published for supported targets.

Code signing and notarization are intentionally out of scope.

---

## SSH and tmux

Raster Nights does not host a public SSH game service in v1. Instead, users install the native binary on their own workstation, server, development machine, or home lab.

Example:

```bash
ssh my-server
tmux new -s raster-nights
raster-nights
```

A fast direct launch may look like:

```bash
raster-nights play loopback --quick
```

tmux or GNU Screen is responsible for preserving a live process after a disconnected SSH client. Raster Nights does not serialize arbitrary mid-game state in v1.

---

## Repository layout

```text
.
├── AGENTS.md
├── README.md
├── apps/
│   ├── terminal/
│   └── web/
├── crates/
│   ├── raster-engine/
│   ├── raster-display/
│   ├── raster-games/
│   ├── raster-storage/
│   ├── raster-audio/
│   └── raster-testkit/
├── website/
├── content/
├── assets/
├── scripts/
└── docs/
```

This repository is a monorepo containing the game system, browser and terminal hosts, website, manuals, fictional canon, editable assets, and release tooling.

---

## Development model

Development occurs directly on `master`.

The project favors:

- focused, reversible commits;
- one complete vertical slice at a time;
- native and browser implementation together;
- deterministic tests;
- simple CI;
- polished tagged releases.

The expected repository validation command is:

```bash
./scripts/check.sh
```

See `docs/DEVELOPMENT.md` for local setup and workflow.

---

## Contributions

Raster Nights is publicly developed but owner-curated.

Narrowly scoped maintenance pull requests may be considered, including:

- verified bug fixes;
- platform compatibility fixes;
- accessibility improvements;
- tests;
- documentation corrections;
- security fixes.

The project does not accept unsolicited:

- new official games;
- fictional studios or lore;
- major features;
- architectural rewrites;
- replacement branding;
- generated code dumps.

See `docs/LICENSING.md` for software, asset, and naming policies.

---

## Documentation map

- `AGENTS.md` — coding-agent rules and repository invariants
- `docs/PRODUCT.md` — complete product requirements
- `docs/ARCHITECTURE.md` — technical architecture and boundaries
- `docs/DESIGN.md` — visual, interaction, accessibility, and copy system
- `docs/CANON.md` — fictional world, timeline, companies, titles, and tone
- `docs/DEVELOPMENT.md` — local setup, commands, workflow, and releases
- `docs/DECISIONS.md` — accepted decisions and rationale
- `docs/LICENSING.md` — software license, reserved creative content, and fork policy
- `docs/ROADMAP.md` — milestone sequence and quality gates
- `docs/plans/` — active implementation plans

---

## License and reserved identity

The intended licensing model is:

- original software source: MPL-2.0;
- general technical documentation: CC BY 4.0;
- logos, artwork, music, fictional manuals, game titles, studio identities, and fictional canon: reserved unless an explicit license states otherwise;
- third-party assets: their respective licenses.

The project name and fictional identity must not be used by forks to imply that they are official Raster Nights releases.

See `docs/LICENSING.md` before redistributing the project.

---

## Creator

Raster Nights is created and curated by **Drilon Reçica**.
