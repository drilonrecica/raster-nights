# AGENTS.md — Raster Nights Coding-Agent Operating Manual

This file is the primary operating manual for AI coding agents working in this repository. It is intentionally explicit because agents must not rely on prior conversations, personal memory, or assumptions about the project.

Raster Nights is a curated retro game system written in Rust. It runs as:

1. a native terminal application, including through normal SSH sessions and inside tmux or GNU Screen; and
2. a browser application compiled to WebAssembly and presented inside a terminal-like display.

The product is presented as software for a fictional 1990s home computer called the **DRX-90**, manufactured by **Reçica Computer Works**, running **R/OS**, with games launched through the **AfterHours** software archive. The project combines playable games, an engineering showcase, and a coherent fictional software ecosystem.

The official catalog and creative direction are curated solely by Drilon Reçica. This is an open-source software project, but it is not a community game platform, plugin ecosystem, or public game SDK.

---

## 1. Authority and document order

Read documents according to the work being performed.

### Always read before making nontrivial changes

1. `AGENTS.md`
2. the active plan in `docs/plans/`
3. `docs/PRODUCT.md`
4. `docs/ARCHITECTURE.md`

### Also read when relevant

- Visual, UX, copy, animation, layout, branding, or accessibility work:
  - `docs/DESIGN.md`
  - `docs/CANON.md`
- Build, testing, local setup, release, or repository workflow:
  - `docs/DEVELOPMENT.md`
- Licensing, assets, names, or redistribution:
  - `docs/LICENSING.md`
- Historical rationale for a technical or product choice:
  - `docs/DECISIONS.md`
- Milestone sequencing:
  - `docs/ROADMAP.md`

### Conflict resolution

When documents appear to conflict:

1. Current explicit task instructions take precedence for the task.
2. Product behavior is governed by `docs/PRODUCT.md`.
3. Technical boundaries are governed by `docs/ARCHITECTURE.md`.
4. Visual and interaction behavior is governed by `docs/DESIGN.md`.
5. Fictional names, dates, companies, tone, and lore are governed by `docs/CANON.md`.
6. Workflow and command conventions are governed by `docs/DEVELOPMENT.md`.
7. The active plan may narrow scope but must not silently contradict permanent documents.
8. Existing code is not automatically authoritative if it conflicts with accepted documentation.

When a requested change intentionally changes an accepted decision, update the relevant permanent document and add or amend an entry in `docs/DECISIONS.md`.

---

## 2. Product mission

Raster Nights should make users think:

> “These are substantial, polished retro games for a computer that never existed—and the same games genuinely run in my terminal and in the browser.”

The project serves three goals simultaneously:

1. **Playable arcade:** The public experience must be enjoyable beyond its technical novelty.
2. **Engineering showcase:** The architecture should demonstrate disciplined Rust, deterministic simulation, clean platform boundaries, and thoughtful release engineering.
3. **Curated open-source project:** Source is public, but the official games, fictional canon, names, art direction, and roadmap remain owner-controlled.

When goals conflict, prioritize in this order:

1. player experience and correctness;
2. native/web behavioral parity;
3. maintainable architecture;
4. fictional presentation and polish;
5. implementation convenience.

---

## 3. Non-negotiable product constraints

Agents must preserve these constraints unless the owner explicitly changes them.

### Platforms

- Native terminal and browser are first-class platforms.
- Features affecting game behavior must be implemented for both hosts from the beginning.
- Native terminal play must work locally, through SSH, and inside tmux.
- The canonical logical display is **100 columns × 36 rows**.
- Native terminals smaller than 100×36 suspend the session and display a resize message.
- Browser rendering scales the full logical grid; gameplay is not cropped to enlarge text.

### Network and privacy

- The installed application makes no outbound network requests in v1.
- No analytics, telemetry, advertising, accounts, cloud saves, update checks, remote content, or automatic diagnostic uploads.
- Settings, high scores, puzzle records, and system state are local.
- Diagnostic reports are local and voluntary.
- Never introduce a network dependency, SDK, tracker, beacon, or hosted API without an explicit accepted decision.

### Catalog ownership

- There is no plugin system.
- There is no downloadable game format.
- There is no third-party game API guarantee.
- Games are compiled into the application.
- Do not add new official games, fictional studios, major lore, or public extension points unless explicitly requested.

### Gameplay

- All official games are single-player in v1.
- No accounts, multiplayer, persistent progression, unlock trees, currency, or cloud-based achievements.
- Local high scores are allowed.
- Three-character tags are entered after qualifying scores.
- Flagship games should be mechanically substantial, not merely demonstrations.
- Games must remain playable without audio and must not communicate critical state through color alone.

### Presentation

- The experience is an original 1990s-inspired home computer, not an imitation of a specific real machine.
- The tone is sincere retro computing with dry, facetious system humor.
- Humor should be occasional and deadpan, not constant or disruptive.
- The product may feel mildly uncanny or melancholic but is not horror.
- The boot, launcher, software loading, manuals, studios, and release chronology are part of the product, not optional decoration.

---

## 4. Architectural invariants

These invariants are mandatory.

1. **Games do not depend on platform APIs.**
   - No Crossterm, Ratzilla, `web-sys`, browser storage, native filesystem access, or terminal state manipulation inside game modules.

2. **Platform input is normalized before reaching games.**
   - Games receive semantic actions such as `MoveLeft`, `RotateClockwise`, `Pause`, or game-specific actions.
   - Games do not inspect browser key codes or Crossterm events.

3. **Authoritative simulation is deterministic.**
   - Fixed simulation ticks.
   - Seeded random number generation.
   - Integer or fixed-point authoritative state where determinism matters.
   - Wall-clock time must not directly drive gameplay.

4. **Rendering does not mutate game state.**
   - Rendering reads state and writes cells to the display abstraction.
   - Animation clocks that affect gameplay belong to simulation state.

5. **Native and web rules are equivalent.**
   - Physics, scoring, randomization, collision, difficulty, and game-over conditions must match.
   - Hosts may differ in input devices, audio implementation, browser-only CRT effects, fullscreen behavior, and storage adapters.

6. **Shared crates do not perform uncontrolled I/O.**
   - Storage, audio, clock, and host events are injected through explicit interfaces.

7. **The 100×36 display is canonical.**
   - Games may use a smaller internal viewport inside the grid.
   - Do not create separate native and browser layouts unless explicitly approved.

8. **Terminal cleanup is safety-critical.**
   - Raw mode, alternate screen, mouse capture, cursor visibility, and input state must be restored after normal exit and panic where reasonably possible.

9. **Game behavior must be testable without a real terminal or browser.**
   - Core simulations and shared rendering must run in ordinary Rust tests.

10. **No hidden product behavior.**
    - No silent network access, telemetry, data collection, or remote execution.

---

## 5. Expected repository structure

The intended monorepo structure is:

```text
raster-nights/
├── AGENTS.md
├── README.md
├── Cargo.toml
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
│   ├── games/
│   ├── localization/
│   ├── manuals/
│   └── fictional-canon/
├── assets/
│   ├── fonts/
│   ├── graphics/
│   ├── music/
│   └── sounds/
├── scripts/
└── docs/
```

Do not create new top-level directories casually. When adding one, document its purpose in `docs/ARCHITECTURE.md` or `docs/DEVELOPMENT.md`.

---

## 6. Agent workflow

For each task:

1. Read the active plan and relevant permanent documents.
2. Inspect the existing implementation before proposing architecture.
3. Identify the smallest coherent vertical slice.
4. Confirm which host-independent code and which host adapters are affected.
5. Implement shared behavior first.
6. Implement native and browser integration together when the feature is platform-facing.
7. Add or update deterministic tests.
8. Add or update rendering snapshots where visual cells change.
9. Run the repository validation commands.
10. Review the diff for unrelated changes.
11. Update documentation only where behavior or decisions changed.
12. Report:
    - what changed;
    - why;
    - tests run;
    - known limitations;
    - any deliberate follow-up left out of scope.

Do not leave placeholder implementations marked as complete.

---

## 7. Scope discipline

Agents must avoid broad opportunistic refactors.

### Do

- change the smallest set of files that delivers the requested behavior;
- preserve public and internal naming consistency;
- add tests close to the behavior being changed;
- use existing abstractions before adding new ones;
- explain new dependencies and place them in the correct host or shared crate;
- keep data-driven balancing values outside code when appropriate;
- preserve deterministic behavior;
- preserve native/browser parity.

### Do not

- redesign unrelated modules;
- add an async runtime “for future use”;
- add an ECS for small games without demonstrated need;
- add a plugin system;
- add backend services;
- add analytics or update checks;
- add a frontend framework without an accepted decision;
- create a new crate for every small concern;
- duplicate logic between web and terminal hosts;
- invent new lore while solving a technical task;
- replace accepted names, dates, or studios;
- add large generated code dumps without review;
- commit generated build outputs;
- make release CI complicated.

---

## 8. Coding standards

### Rust

- Use stable Rust.
- Prefer clear domain types over primitive-value ambiguity.
- Use `Result` for recoverable errors.
- Use typed error enums in library crates.
- Add context at application boundaries.
- Avoid panics in normal user-controlled paths.
- Panics are acceptable for violated internal invariants in tests or impossible states, but document the invariant.
- Use exhaustive matches where they improve safety.
- Keep functions focused and state transitions explicit.
- Avoid hidden global mutable state.
- Avoid unnecessary cloning in hot loops, but do not sacrifice clarity for premature micro-optimization.
- Do not use `unsafe` without an explicit decision and a documented safety argument.
- Keep platform `cfg` blocks confined to host or adapter code.
- Avoid float values in authoritative deterministic state unless proven safe and included in cross-platform golden testing.

### State machines

Boot, launcher, game session, pause, game over, tag entry, shell, and shutdown should be modeled as explicit states. Avoid scattered Boolean flags that permit impossible combinations.

### Data formats

- User-facing settings: TOML.
- User-facing records: JSON unless another documented format is better.
- Authored balancing and content: TOML, JSON, or other human-readable structured files.
- Internal replay/debug data may use a compact binary format, but it is not a stable public interface.
- All stored data must have a format version.
- Writes must be atomic where possible.
- Corrupt files should be preserved or renamed before recovery.

### Dependencies

A dependency is acceptable when it removes substantial risk or boilerplate and is actively maintained under a compatible license.

Before adding one:

1. confirm which crate owns it;
2. disable unused default features;
3. document why it is needed;
4. avoid adding it to shared crates when only one host needs it;
5. verify it does not introduce network behavior, telemetry, or incompatible licensing.

---

## 9. Input rules

- The engine owns held-key state; do not rely on OS keyboard repeat.
- Track pressed, held, released, and held duration.
- Arrow keys are primary navigation.
- `H`, `J`, `K`, and `L` navigate globally when text entry is inactive.
- `Esc` behavior is consistent:
  - gameplay: pause;
  - submenu: back;
  - detail screen: catalog;
  - launcher: system menu;
  - command line: clear current input.
- `Ctrl+C` opens a short safe-exit confirmation; a second `Ctrl+C` exits immediately.
- Browser focus loss pauses immediately and requires explicit resume.
- Terminal resize below minimum suspends simulation and timers.
- Native mouse may be used for menus and Bureau 9, but keyboard control remains complete.
- Web launcher supports keyboard and mouse.
- Touch controls look like DRX-90 hardware, not modern translucent mobile-game controls.

---

## 10. Rendering rules

- Canonical grid: 100×36.
- Canonical glyphs should be single-cell and tested across supported terminals and Ratzilla.
- Avoid ambiguous-width Unicode.
- Do not use glyphs that render as emoji by default.
- Critical information must not rely on color alone.
- Games render through the project display façade.
- Browser CRT effects are presentation-only and must not affect game geometry.
- Native terminal rendering must remain attractive without CRT effects.
- Rendering snapshots use structured cell data plus a readable character-grid representation.
- Do not create a second graphical sprite renderer for web gameplay.

---

## 11. Determinism rules

Every real-time game uses a fixed simulation rate, initially 60 ticks per second.

A deterministic run is defined by:

```text
game rules revision
+ mode
+ seed
+ ordered actions tagged with simulation ticks
= authoritative final state
```

State hashes must exclude:

- renderer-only interpolation;
- audio playback state;
- wall-clock timestamps;
- platform-specific focus events after they are normalized;
- floating-point projection values used only for presentation.

Golden runs should be executable in native tests and WebAssembly tests and should produce the same final authoritative hash.

---

## 12. Testing expectations

At minimum, changes should include the relevant subset of:

- unit tests for game rules;
- deterministic golden runs;
- rendering buffer snapshots;
- storage migration tests;
- host input mapping tests;
- terminal cleanup tests where practical;
- browser integration smoke tests;
- content validation tests.

The standard validation command should eventually be:

```bash
./scripts/check.sh
```

Until that script exists, run the commands specified in `docs/DEVELOPMENT.md`.

Do not weaken or delete tests merely to make a change pass unless the accepted behavior changed and the documentation is updated.

---

## 13. Performance expectations

Performance matters most in Afterline 99 and Mnemonic Nullway, but avoid premature optimization.

General budgets:

- simulation update should normally remain below 1 ms;
- complex game simulation should normally remain below 2 ms;
- shared cell rendering should normally remain below 4 ms;
- browser gameplay should aim for 60 FPS;
- native terminal gameplay should remain playable at 30 FPS and target 60 where supported;
- the Wasm core should remain reasonably small;
- startup should render promptly, even when theatrical boot animation lasts several seconds.

When optimizing:

1. measure first;
2. preserve determinism;
3. preserve readability;
4. add a benchmark or regression test when practical.

---

## 14. Fiction and copy rules

Before adding fictional text, read `docs/CANON.md`.

Key rules:

- Use the official dates in `DD.MM.YYYY`.
- The fictional commercial era is 05.10.1993–31.12.1999.
- The current real system date may be displayed and may exceed the fictional warranty period.
- NUL means **Nonessential Utility Layer**.
- NUL is funny, helpful, slightly insubordinate, and used sparingly.
- Personal-source names are transformed into fictional brands; never add real family relationships, addresses, or biographical details.
- Humor is dry and system-like.
- Avoid constant jokes, memes, modern internet slang, and parody of a single real platform.
- Never imply the application accessed private data that it did not access.
- Do not make fictional error messages resemble real destructive operations unless the effect is harmless and clearly contained.

---

## 15. Documentation update rules

Update permanent documents only when accepted behavior or architecture changes.

- Product behavior changed → `docs/PRODUCT.md`
- Technical boundary changed → `docs/ARCHITECTURE.md`
- Visual or interaction rule changed → `docs/DESIGN.md`
- Canon changed → `docs/CANON.md`
- Workflow or commands changed → `docs/DEVELOPMENT.md`
- License or asset policy changed → `docs/LICENSING.md`
- Significant decision changed → `docs/DECISIONS.md`
- Milestone sequencing changed → `docs/ROADMAP.md`
- Current task progress changed → active plan only

Do not duplicate configuration values that already exist in machine-readable source unless the value is a product invariant.

---

## 16. Contribution and ownership rules

Maintenance contributions may be accepted, but agents should not design workflows around external contributors.

Acceptable external changes may include:

- bug fixes;
- compatibility fixes;
- accessibility improvements;
- tests;
- documentation corrections;
- security fixes.

The following remain owner-controlled:

- official games;
- gameplay direction;
- fictional names;
- fictional studios;
- canon;
- logos;
- music;
- artwork;
- major architecture;
- roadmap.

Do not add templates, plugin manifests, contributor game SDKs, or automated game discovery.

---

## 17. Release rules

The release process must remain simple.

A normal release should be:

1. ensure `master` passes checks;
2. update version and release notes;
3. create a version tag;
4. CI builds the supported archives and browser bundle;
5. CI generates SHA-256 checksums;
6. publish a GitHub release;
7. deploy the static website;
8. update the Homebrew tap.

No code signing, notarization, paid certificates, elaborate multi-stage promotion system, or automatic deployment of every `master` commit.

Tagged releases must be polished. `master` may contain in-progress work, but it should remain buildable whenever practical.

---

## 18. Definition of done

A feature is not done until:

- behavior matches the active plan and permanent specifications;
- native and web implications are handled;
- deterministic behavior is preserved;
- relevant tests pass;
- user-facing errors are clear;
- no new network activity or privacy issue was introduced;
- terminal cleanup remains safe;
- accessibility implications were considered;
- documentation is updated where necessary;
- no unrelated changes remain in the diff.

When only part of a feature is implemented, state that clearly and leave the active plan checklist accurate.

---

## 19. Current public naming

Use these exact names unless a later accepted decision changes them:

- Public project: **Raster Nights**
- Manufacturer: **Reçica Computer Works**
- Computer: **DRX-90**
- Operating system: **R/OS**
- Arcade environment: **AfterHours**
- System entity: **NUL — Nonessential Utility Layer**
- Flagship games:
  - **Signal Stack**
  - **Bureau 9**
  - **Mnemonic Nullway**
  - **Afterline 99**
- Bonus games:
  - **Loopback**
  - **Hazard Registry**
  - **Relay Breaker**
- Hidden game:
  - **Packet Sweep**

Use ASCII-safe forms in executable names, package names, file paths, and identifiers. Preserve diacritics in user-facing branding where the environment supports them.

---

## 20. Final agent reminder

Raster Nights should not feel like a collection of programming tutorials wrapped in a theme. It should feel like a real, lost software archive whose unusual technical implementation happens to be excellent.

Prefer a smaller, finished, coherent feature over a broad, partially integrated one.
