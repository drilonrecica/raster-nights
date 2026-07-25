# Raster Nights Decision Log

**Status:** Living record of accepted decisions  
**Purpose:** Prevent repeated re-litigation and accidental reversal of deliberate choices

Each entry records context, decision, consequences, and conditions for reconsideration. This file is not a task list. New decisions should be added when they materially affect product behavior, architecture, workflow, licensing, or scope.

---

## ADR-001 — One product with three goals

**Status:** Accepted

### Context

Raster Nights is intended to be a playable arcade, an engineering showcase, and an open-source project. These goals can conflict.

### Decision

Treat all three as real goals, with priority:

1. player experience and correctness;
2. native/web parity;
3. maintainable architecture;
4. fictional polish;
5. implementation convenience.

### Consequences

The project cannot excuse shallow games as technical demos. Architecture cannot dominate the public experience.

### Reconsider when

Only if the owner intentionally repositions the project.

---

## ADR-002 — Original fictional computer and operating system

**Status:** Accepted

### Decision

Use an original machine inspired by multiple 1990s systems rather than reproducing a real computer.

- Manufacturer: Reçica Computer Works
- Machine: DRX-90
- OS: R/OS
- Arcade: AfterHours

### Consequences

The project may use familiar era conventions while developing distinct branding and canon.

---

## ADR-003 — Public project name is Raster Nights

**Status:** Accepted, pending formal name clearance

### Decision

Use Raster Nights as public project name, repository name, and canonical executable.

### Consequences

The fictional AfterHours name remains in-universe. `rnights` may be a convenience alias.

### Reconsider when

Trademark, package, domain, or major naming conflict is discovered.

---

## ADR-004 — Fixed 100×36 logical display

**Status:** Accepted

### Context

Responsive terminal layouts multiply design and testing work across every game.

### Decision

All machine screens use a canonical 100×36 logical grid. Native terminals below the minimum suspend. Browser scales the entire grid.

### Consequences

Games have predictable geometry and parity. Very small terminals and portrait mobile screens may be unsupported for real-time games.

### Reconsider when

A proven accessibility or platform need cannot be solved through scaling or specialized controls.

---

## ADR-005 — Native and browser from the beginning

**Status:** Accepted

### Decision

Every foundational feature and official game is implemented for native terminal and browser together.

### Consequences

Early work is slower than terminal-only prototyping, but avoids late architectural divergence.

---

## ADR-006 — Shared deterministic game core

**Status:** Accepted

### Decision

Use fixed-step simulation, seeded RNG, normalized actions, and authoritative integer/fixed-point state.

### Consequences

Runs can be reproduced and compared across native and Wasm. Rendering and audio remain non-authoritative.

---

## ADR-007 — Ratatui cell model behind a project façade

**Status:** Accepted

### Decision

Use Ratatui core cell/buffer primitives underneath a Raster Nights display abstraction. Games do not use full Ratatui widgets directly.

### Consequences

The project gains tested terminal-cell infrastructure without exposing games to host-specific UI concerns.

### Reconsider when

Ratatui core becomes unsuitable or creates a proven browser limitation.

---

## ADR-008 — Ratzilla WebGL2 with Canvas fallback

**Status:** Accepted

### Decision

Use Ratzilla WebGL2 as preferred browser cell renderer and Canvas as fallback.

### Consequences

The browser can update full grids efficiently. Canonical glyphs must be tested and restricted to reliable single-cell symbols.

### Reconsider when

A renderer becomes unmaintained, incompatible, or unable to meet quality targets.

---

## ADR-009 — Astro website with minimal TypeScript

**Status:** Accepted

### Decision

Use Astro for the static public website. Mount the Rust/Wasm DRX-90 as one interactive region. Avoid React, Vue, Svelte, Yew, or Leptos in v1.

### Consequences

Most site content remains static and accessible. JavaScript remains limited to browser integration.

### Reconsider when

A concrete interactive website requirement exceeds Astro’s model.

---

## ADR-010 — Synchronous native loop, no Tokio in v1

**Status:** Accepted

### Decision

Use a synchronous Crossterm polling loop and standard time primitives. Do not add Tokio to the native game application.

### Consequences

Lower complexity and dependency weight.

### Reconsider when

Networking or substantial concurrent I/O enters the native application.

---

## ADR-011 — One curated games crate

**Status:** Accepted

### Decision

Place official games as modules in one `raster-games` crate rather than one crate per game.

### Consequences

Less manifest and feature complexity. Flagship modules may still have many internal files.

### Reconsider when

Compile times, dependency isolation, or ownership boundaries become demonstrably problematic.

---

## ADR-012 — No plugins or public contribution framework

**Status:** Accepted

### Decision

Games are compiled into the application and registered explicitly. No plugin discovery, scripting, runtime loading, or public SDK.

### Consequences

Simpler Wasm builds, security model, testing, and versioning.

### Reconsider when

Only through a deliberate product repositioning.

---

## ADR-013 — Owner-curated official catalog

**Status:** Accepted

### Decision

No contributed official games. Maintenance fixes may be accepted, but official games, canon, major features, and creative direction remain owner-controlled.

### Consequences

Open source does not mean open creative governance.

---

## ADR-014 — No network activity in v1

**Status:** Accepted

### Decision

The installed application makes no outbound network requests.

### Consequences

No global leaderboard, update check, remote manual, analytics, cloud save, or remote content.

### Reconsider when

A specific optional online feature provides sufficient value and can remain isolated.

---

## ADR-015 — No analytics, ever by default

**Status:** Accepted

### Decision

Do not add analytics or telemetry as a normal product feature.

### Consequences

Success is measured through voluntary public feedback and product quality, not behavioral tracking.

---

## ADR-016 — Human-readable local persistence

**Status:** Accepted

### Decision

Use TOML for settings and JSON for local records where practical. Version schemas and write atomically.

### Consequences

Users can inspect and back up data. The application must handle corruption gracefully.

### Reconsider when

Data volume or transactional requirements genuinely exceed file storage.

---

## ADR-017 — No arbitrary live-game saves in v1

**Status:** Accepted

### Decision

Persist committed records and stable settings, not entire in-memory game states.

### Consequences

Broken SSH sessions end current runs unless tmux or Screen preserves the process.

---

## ADR-018 — Browser power-on is explicit

**Status:** Accepted

### Decision

Do not auto-boot on page load. Use `POWER ON DRX-90`.

### Consequences

Focus and audio activation are predictable, and normal website content remains available.

---

## ADR-019 — Cold and warm boot, no permanent boot disable

**Status:** Accepted

### Decision

Provide cold and warm boot. Any input skips. Quiet mode shortens ceremony. Direct `--quick` launch bypasses most presentation.

### Consequences

Product identity remains present without making repeat use tedious.

---

## ADR-020 — Hybrid launcher and shell

**Status:** Accepted

### Decision

Use a menu-driven archive as primary navigation with an optional miniature R/OS shell.

### Consequences

Ordinary users discover games easily; power users gain commands and lore.

---

## ADR-021 — Full-screen games

**Status:** Accepted

### Decision

Games use the full canonical display. The persistent OS frame does not permanently consume game space.

### Consequences

Launcher, pause, and game-over screens provide system framing.

---

## ADR-022 — Original genre interpretations, not branded clones

**Status:** Accepted

### Decision

Use familiar genre mechanics while creating original names, art, scoring, fiction, vehicles, obstacles, and modes.

### Consequences

The project can be approachable without copying protected branding or assets.

---

## ADR-023 — Flagship and bonus catalog

**Status:** Accepted

### Decision

v1 target:

Flagships:

- Signal Stack
- Bureau 9
- Mnemonic Nullway
- Afterline 99

Bonus:

- Loopback
- Hazard Registry
- Relay Breaker

Hidden:

- Packet Sweep

### Consequences

Eight total playable titles provide variety while preserving quality.

---

## ADR-024 — Signal Stack is the first vertical-slice game

**Status:** Accepted

### Decision

Build Signal Stack before Bureau 9, Mnemonic Nullway, and Afterline 99.

### Consequences

The first game exercises real-time timing, input, rendering, score, pause, persistence, and determinism.

---

## ADR-025 — Afterline 99 is the primary showcase flagship

**Status:** Accepted

### Decision

Afterline 99 receives the greatest eventual presentation and technical effort.

### Consequences

It should demonstrate that a terminal-cell game can deliver a convincing pseudo-3D racer with mechanical depth.

---

## ADR-026 — Shared fictional V-SCAPE projection technology

**Status:** Accepted

### Decision

Mnemonic Nullway and Afterline 99 share a fictional and potentially technical projection system developed by Vranidoll Signal Works.

### Consequences

Projection utilities may be shared. Gameplay abstractions must remain separate when they differ.

---

## ADR-027 — NUL is Nonessential Utility Layer

**Status:** Accepted

### Decision

Canonical acronym:

> NUL — Nonessential Utility Layer

NUL is a small resident system personality: helpful, dry, and slightly insubordinate.

### Consequences

Other expansions may appear only as jokes or rumors.

---

## ADR-028 — Real local clock, fictional software chronology

**Status:** Accepted

### Decision

R/OS displays the user’s real local date and time. Games have fictional release dates from 05.10.1993 through 31.12.1999.

### Consequences

The machine may joke that the real date exceeds the certified operating period.

---

## ADR-029 — Three-character score tags

**Status:** Accepted

### Decision

Users enter a three-character tag after each qualifying score. The previous tag is preselected but submission remains explicit.

### Consequences

No player account or profile model is needed.

---

## ADR-030 — Local scoreboards per OS user/browser profile

**Status:** Accepted

### Decision

Store records per local OS user or browser profile.

### Consequences

No elevated permissions or cross-user contention. Shared server scores may be considered later as an explicit configuration.

---

## ADR-031 — Accessibility profiles do not automatically invalidate scores

**Status:** Accepted

### Decision

Presentational accessibility options preserve canonical eligibility. Mechanical assistance creates an assisted rules profile.

### Consequences

Accessibility is not framed as cheating, while records remain comparable.

---

## ADR-032 — Web audio optional, native silent by default

**Status:** Accepted

### Decision

Browser supports original music and effects after explicit user interaction. Native terminal is silent by default. Terminal bell is not a gameplay audio system.

### Consequences

All important cues require visual equivalents.

---

## ADR-033 — Original audio and editable source assets

**Status:** Accepted

### Decision

Prefer original tracker/FM-inspired music and maintain editable source assets in open formats when practical.

### Consequences

Avoid generic asset-library identity and opaque proprietary source formats.

---

## ADR-034 — Localization-ready, English-only v1

**Status:** Accepted

### Decision

Use stable string IDs and avoid rule dependence on English. Ship English only in v1.

### Consequences

Layouts should tolerate some expansion, but complex-script support is not yet guaranteed.

---

## ADR-035 — Direct development on `master`

**Status:** Accepted

### Decision

Owner development occurs directly on `master`. Temporary branches are optional.

### Consequences

CI and focused commits matter. Published history should not be force-pushed.

---

## ADR-036 — Monorepo

**Status:** Accepted

### Decision

Keep Rust apps, shared crates, website, content, manuals, assets, and release scripts in one repository.

### Consequences

Cross-platform and content changes can be coordinated in one commit.

---

## ADR-037 — Informal issues and high-level roadmap

**Status:** Accepted

### Decision

Keep public issues enabled and informal. Publish direction without dates or exhaustive commitments.

### Consequences

The project remains approachable without maintenance bureaucracy.

---

## ADR-038 — Tagged releases are polished

**Status:** Accepted

### Decision

Public pre-1.0 tags must feel intentional and playable. `master` may be in progress.

### Consequences

No placeholder-heavy public releases.

---

## ADR-039 — Simple CI and release process

**Status:** Accepted

### Decision

CI runs format, lint, tests, native build, Wasm build, and website build. Release tags build archives, checksums, GitHub release, website, and Homebrew update.

### Consequences

No elaborate matrix, staging system, or automated deployment of every commit.

---

## ADR-040 — No signing or notarization

**Status:** Accepted

### Decision

Do not use Apple notarization, Developer ID signing, Windows signing, or paid certificates.

### Consequences

Users may see OS warnings. Publish checksums and clear installation instructions.

---

## ADR-041 — Initial required native platforms

**Status:** Accepted

### Decision

0.1 officially supports:

- macOS Apple Silicon;
- macOS Intel;
- Fedora-compatible Linux x86-64.

### Consequences

Windows, Linux ARM, musl, RPM repositories, and other package channels are deferred.

---

## ADR-042 — Homebrew tap in 0.1

**Status:** Accepted

### Decision

Provide a Homebrew tap for initial release.

### Consequences

The release process must publish stable archives and checksums consumable by the formula.

---

## ADR-043 — Software code under MPL-2.0

**Status:** Provisional accepted recommendation; legal review advised before major commercial use

### Decision

License original software source under MPL-2.0.

### Consequences

Direct modifications to covered files remain under MPL, while broader combinations are possible.

### Reconsider when

Legal review identifies a better fit or business model changes.

---

## ADR-044 — Creative identity separately reserved

**Status:** Accepted policy direction

### Decision

Keep logos, artwork, music, fictional manuals, titles, studios, and canon outside the code license unless explicitly licensed.

### Consequences

Forks may use licensed code but must replace reserved identity and assets unless permitted.

---

## ADR-045 — Substantial fiction in 0.1

**Status:** Accepted

### Decision

0.1 launches with meaningful manuals, studio profiles, system files, reviews, and NUL interactions.

### Consequences

The first release is already a product world, not a plain technical shell.

---

## ADR-046 — Scope-cut order

**Status:** Accepted

### Decision

Cut in this order when needed:

1. full soundtrack;
2. elaborate CRT;
3. advanced racer/runner touch;
4. deep shell;
5. extensive lore;
6. polished remapping UI;
7. attract modes;
8. extra challenges.

### Consequences

Core parity, safety, determinism, gameplay, privacy, and identity remain protected.

---

## ADR-047 — No fixed release cadence

**Status:** Accepted

### Decision

Release when a milestone satisfies quality gates, not on a calendar.

### Consequences

Roadmap contains sequencing without dates.

---

## ADR-048 — Success is quality and real use, not a star target

**Status:** Accepted

### Decision

Judge success by installability, browser reliability, SSH/tmux play, enjoyable games, coherent identity, and engineering recognition.

### Consequences

Do not optimize product decisions for superficial repository metrics.

---

## ADR-049 — Post-1.0 priority is more games

**Status:** Accepted

### Decision

After stability, prioritize substantial new games, then deeper lore, polish, mobile, and optional online experiments.

### Consequences

Do not convert the project into an account service or plugin platform merely because it gains attention.

---

## Updating this log

When adding a decision:

```markdown
## ADR-XXX — Title

**Status:** Proposed | Accepted | Superseded | Rejected

### Context

### Decision

### Consequences

### Reconsider when
```

If a decision is superseded, preserve it and link to the new entry.
