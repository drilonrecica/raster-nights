# Raster Nights Development Guide

**Status:** Working development process  
**Audience:** Owner, coding agents, maintainers, and reviewers

This document explains how to work in the Raster Nights monorepo: setup, commands, branch policy, validation, content authoring, debugging, testing, and releases.

The process is intentionally lightweight. Raster Nights is owner-developed directly on `master`, uses simple CI, and favors focused vertical slices over elaborate project-management infrastructure.

---

## 1. Development philosophy

- Build one coherent vertical slice at a time.
- Implement shared logic once.
- Integrate native terminal and browser behavior together.
- Keep `master` understandable and preferably buildable.
- Use tests for deterministic behavior and state transitions.
- Use manual playtesting for feel and visual quality.
- Keep CI short enough to understand.
- Tag only polished releases.
- Do not add operational complexity unless the product requires it.

AI-generated code is acceptable, but generated code must be:

- reviewed;
- integrated with existing architecture;
- tested;
- formatted;
- licensed appropriately;
- understandable enough to maintain.

Large unreviewed generated changes are not acceptable.

---

## 2. Branch and commit policy

### Default branch

```text
master
```

Development happens directly on `master`.

Temporary local branches are acceptable for risky experiments, but they are not mandatory and should not create a review bureaucracy.

### History rules

- Do not force-push published `master`.
- Keep commits focused and reversible.
- Prefer one coherent behavior change per commit.
- Do not mix formatting of unrelated files with feature work.
- Do not commit broken generated artifacts.
- Tag only commits that pass the release checklist.

### Commit messages

Use direct descriptive messages:

```text
Add deterministic simulation clock
Implement native terminal restoration guard
Render DRX-90 cold boot in both hosts
Add Signal Stack seven-packet bag
Persist local score records atomically
```

Avoid vague messages:

```text
update
fix stuff
AI changes
WIP final
```

---

## 3. Toolchain

Use stable Rust.

The workspace minimum supported Rust version is 1.90. This is the lowest
version supported by the selected Ratzilla 0.3 browser-rendering dependency
chain. Normal development and release checks use the current stable toolchain;
CI also checks the declared minimum version.

Likely tools:

- Rust toolchain and Cargo
- `wasm32-unknown-unknown` target
- `wasm-pack` for browser package generation and headless Wasm tests
- Node.js for the website toolchain
- Astro for the static website
- a Wasm bundling/dev command selected by the web application
- Git
- Homebrew for testing macOS packaging when available

Do not install global tools unnecessarily when a project-local command or Cargo subcommand can be used.

---

## 4. Initial setup

Illustrative setup after the repository exists:

```bash
git clone <repository-url>
cd raster-nights

rustup toolchain install stable
rustup target add wasm32-unknown-unknown

cargo build --workspace
```

Website setup:

```bash
cd website
npm install
npm run build
```

Exact commands must be kept synchronized with actual manifests and scripts.

---

## 5. Expected common commands

The repository should provide simple scripts rather than requiring agents to remember many command combinations.

### Full validation

```bash
./scripts/check.sh
```

Expected checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
wasm-pack build apps/web --target web --no-pack --out-dir ../../website/public/wasm
wasm-pack test --headless --firefox crates/raster-games
npm --prefix website run build
```

Generated files under `website/public/wasm/` are ignored build output. Adapt
package names to the actual workspace, but keep `wasm-pack` as the single
Rust-to-browser packaging path and keep the script authoritative.

### Native application

```bash
cargo run -p raster-terminal
```

Examples:

```bash
cargo run -p raster-terminal -- display-test
```

The current CLI accepts the normal launcher and `display-test`. Direct game
launch, quiet mode, and authored-content validation are planned 0.1 interfaces,
not implemented commands.

### Web application and website

Build the Wasm package before starting Astro:

```bash
wasm-pack build apps/web \
  --target web \
  --no-pack \
  --out-dir ../../website/public/wasm

npm --prefix website ci
npm --prefix website run dev
```

Astro reports the local URL, normally `http://localhost:4321`. Rebuild the Wasm
package after Rust changes. A combined watch script remains a future workflow
improvement.

### Formatting

```bash
cargo fmt
```

No JavaScript formatter is currently configured. Add one only when it removes
more maintenance work than it creates.

### Focused testing

```bash
cargo test -p raster-games signal_stack
cargo test -p raster-storage
cargo test -p raster-engine app::tests
```

### Benchmarks

Add only when useful:

```bash
cargo bench -p raster-games
```

---

## 6. Local run profiles

### Standard native run

```bash
cargo run -p raster-terminal
```

Quiet mode, direct deterministic seeds, and debug overlays are planned 0.1
development interfaces. They are not accepted by the current CLI.

### Temporary isolated data

Development should support a temporary data directory:

```bash
RASTER_NIGHTS_DATA_DIR=/tmp/raster-nights-dev \
  cargo run -p raster-terminal
```

This avoids corrupting real local scores during tests.

---

## 7. Monorepo conventions

### `apps/`

Only host applications.

### `crates/`

Reusable internal Rust crates.

### `website/`

Astro site, public content integration, and the browser app mount.

### `content/`

Structured authored content:

- puzzle catalogs;
- manuals;
- lore;
- game tuning;
- route and section data;
- localization catalogs.

### `assets/`

Editable and runtime media:

- logos;
- graphics;
- music source;
- sound source;
- exports;
- licensed fonts.

### `scripts/`

Small understandable scripts:

- checking;
- web development;
- content validation;
- release packaging;
- Homebrew formula generation if needed.

Avoid scripts that hide large amounts of opaque logic.

---

## 8. Adding an internal game

This is not a public plugin process. It is an owner workflow.

### Expected steps

1. Add a module under `crates/raster-games/src/`.
2. Define its descriptor and modes.
3. Implement deterministic state and actions.
4. Implement rendering through `raster-display`.
5. Emit semantic audio events.
6. Add tests.
7. Add content data under `content/games/`.
8. Add manual and canon entries.
9. Add one explicit registry entry.
10. Add launcher and detail-screen metadata.
11. Verify native and web behavior.
12. Add golden runs and rendering snapshots.

### Game module structure

A small bonus game may use:

```text
loopback/
├── mod.rs
├── game.rs
├── render.rs
└── tests.rs
```

A flagship may use:

```text
afterline_99/
├── mod.rs
├── game.rs
├── craft.rs
├── race.rs
├── track.rs
├── projection.rs
├── collision.rs
├── scoring.rs
├── render.rs
└── tests.rs
```

Do not create one crate per game unless an accepted architecture decision changes the current approach.

---

## 9. Content authoring

### Structured content

Prefer human-readable data for:

- vehicle values;
- route sections;
- runner segments;
- puzzle catalogs;
- challenges;
- palettes;
- manuals;
- fictional company profiles.

### Code-owned rules

Keep in Rust:

- state transitions;
- scoring algorithms;
- collision rules;
- deterministic randomization;
- invariants;
- value validation logic.

### Validation command

The planned 0.1 content pipeline should provide:

```bash
raster-nights validate-content
```

or an equivalent build/test command.

Validation should fail on:

- duplicate IDs;
- missing references;
- invalid dates;
- invalid value ranges;
- missing assets;
- unsupported glyphs;
- overlong fixed-layout strings;
- inconsistent fictional credits;
- invalid puzzles;
- impossible route transitions.

### Fictional writing

Before authoring, read `docs/CANON.md`.

Never invent real biographical claims from the personal source names used by the fictional companies.

---

## 10. Testing workflow

### Before implementing

Identify:

- authoritative state;
- inputs;
- expected transitions;
- edge cases;
- host-specific adapters;
- persistence effects;
- rendering snapshots.

### During implementation

Add tests alongside behavior, not only after the entire feature.

### Before commit

Run focused tests, then full checks for shared or cross-platform changes.

### Before tag

Run:

- full check script;
- native manual QA;
- browser manual QA;
- supported target build;
- content validation;
- storage migration tests;
- deterministic golden runs;
- release packaging;
- Homebrew install test where practical.

---

## 11. Rendering snapshot workflow

Snapshots should include:

1. readable character grid;
2. style information;
3. canonical grid dimensions;
4. semantic snapshot name.

Examples:

```text
boot/cold-start-initial
launcher/featured-signal-stack-selected
signal-stack/standard-mid-game
signal-stack/game-over-record
system/resize-suspended
```

Snapshot changes require visual review. Do not automatically accept all snapshots after a broad change.

---

## 12. Deterministic golden-run workflow

A golden run should declare:

- game;
- rules revision;
- mode;
- seed;
- action sequence by tick;
- expected score;
- expected status;
- expected state hash.

When behavior changes intentionally:

1. explain why;
2. update product or decision documents if required;
3. update rules revision;
4. regenerate the golden expectation;
5. inspect the behavior manually.

Do not casually update hashes because tests failed.

---

## 13. Debugging tools

Recommended development commands:

```text
display-test
validate-content
diagnostics
play <game> --seed <seed>
play <game> --debug-overlay
play <game> --show-hitboxes
play <game> --show-state-hash
```

These are target devtools interfaces, not commands supported by the current
First Signal CLI. At present, use Rust tests and `display-test`.

Devtools may show:

- simulation tick;
- render timing;
- seed;
- current state hash;
- authoritative positions;
- collision bounds;
- generated section ID;
- input actions;
- storage path;
- renderer backend.

Cheat-like options must disable score submission and should be absent from official release builds unless harmless.

---

## 14. Persistence development

### Test data isolation

Tests use in-memory storage or temporary directories.

### Migration discipline

Every persisted schema has a version.

A migration test should include:

- old valid data;
- malformed data;
- missing fields;
- unknown fields where format permits;
- incompatible future version;
- partial write recovery.

### Atomic writes

Native writes should use temporary files and rename.

Browser writes should surface quota and serialization failures without losing unrelated state.

### Manual inspection

Because settings and scores are human-readable, developers should inspect generated files during milestone QA.

---

## 15. Terminal safety development

Terminal lifecycle changes require special care.

Manual cases:

- normal shutdown;
- `Esc` to system menu and shutdown;
- first `Ctrl+C`;
- repeated `Ctrl+C`;
- panic in gameplay;
- panic during rendering;
- resize below minimum;
- SSH disconnect where testable;
- tmux detach/reattach;
- startup failure after partial initialization.
- enhanced keyboard input and release events;
- compatibility-mode hold expiry;

After each case verify:

- cursor visible;
- raw mode disabled;
- alternate screen exited;
- mouse capture disabled;
- keyboard enhancement flags popped;
- line wrapping and any other modified input mode restored;
- shell input echo restored;
- colors reset.

A terminal cleanup regression blocks release.

The locally automated PTY subset runs through tmux:

```bash
./scripts/test-terminal-cleanup.sh
```

It verifies terminal settings after normal exit and an intentional debug-build
panic, then exercises undersize suspension and explicit resize recovery inside
tmux. SSH disconnect behavior remains a manual environment-dependent check.

---

## 16. Browser development

Test:

- power-on focus;
- focus loss pause;
- tab hidden pause;
- explicit resume;
- WebGL2 backend;
- Canvas fallback;
- audio disabled first visit;
- audio enabled after interaction;
- local records;
- fullscreen;
- mouse;
- touch where relevant;
- portrait rotate-device behavior;
- responsive scaling;
- unsupported small viewport message;
- browser back/navigation behavior.
- semantic mirror focus and actions for supported screens.

Do not require a complex browser matrix for every commit. Use one primary browser in daily development and broader smoke tests before releases.

---

## 17. Website content workflow

The website is static and deployed only for deliberate releases.

Do not deploy every `master` commit automatically.

Content changes should preserve:

- readable semantic HTML;
- project-first explanation;
- browser play;
- installation instructions;
- SSH/tmux message near the hero;
- privacy statement;
- licensing clarity.

The machine experience should not prevent visitors from reading normal documentation.

---

## 18. CI policy

CI should be short and legible.

### On push to `master`

Recommended:

- format;
- Clippy;
- workspace tests;
- workspace build;
- Wasm build;
- website build.

### Optional lightweight platform checks

- macOS compile on `master` or tags;
- Linux primary execution;
- no mandatory Windows matrix for 0.1.

### Release tag

- supported native archives;
- web bundle;
- checksums;
- GitHub release;
- static site deploy;
- Homebrew tap update.

No:

- signing;
- notarization;
- certificate management;
- elaborate staging environments;
- nightly release pipeline;
- automatic public deployment of every commit.

---

## 19. Release versioning

Real project uses semantic versioning:

```text
0.1.0
0.2.0
1.0.0
```

Fictional versions are presentation metadata:

```text
R/OS 3.11
Signal Stack 1.4
V-SCAPE 3.7
```

Do not use fictional versions in Cargo package versioning or technical compatibility logic.

Game data uses internal numeric revisions where needed.

---

## 20. Simple release procedure

A target release procedure:

1. Ensure `master` is clean.
2. Run `./scripts/check.sh`.
3. Perform manual QA checklist.
4. Update real version metadata.
5. Update release notes and roadmap.
6. Confirm `LICENSE`, `NOTICE`, `ASSET-LICENSES.md`, `TRADEMARKS.md`, and
   `DOCUMENT-LICENSES.md` are accurate.
7. Commit release preparation.
8. Tag:
   ```bash
   git tag v0.1.0
   git push origin master --tags
   ```
9. CI builds:
   - macOS Apple Silicon archive;
   - macOS Intel archive;
   - Linux x86-64 GNU archive;
   - browser bundle;
   - SHA-256 checksums.
10. CI creates draft or final GitHub release.
11. Deploy website.
12. Update Homebrew tap.
13. Verify installation from published artifacts.

No code signing or notarization.

---

## 21. Homebrew

The Homebrew formula should:

- install the published archive;
- verify SHA-256;
- expose `raster-nights`;
- optionally add `rnights` alias if practical;
- avoid building the entire workspace from source unless necessary;
- be maintained from a dedicated tap repository if preferred.

Homebrew release automation should remain a small script or clear workflow step.

---

## 22. Issue handling

Issues remain informal.

Useful bug information:

- Raster Nights version;
- operating system;
- terminal and `$TERM`;
- SSH/tmux status;
- terminal size;
- steps;
- expected behavior;
- actual behavior;
- screenshot;
- sanitized local diagnostics.

Do not create a large label taxonomy or bot workflow.

---

## 23. Dependency updates

Update dependencies deliberately.

For each meaningful update:

- review changelog;
- confirm license;
- run checks;
- test native and browser;
- inspect binary/Wasm size if relevant;
- verify no new default network behavior;
- avoid broad dependency churn during release stabilization.

Do not update all dependencies automatically immediately before a release.

---

## 24. Documentation workflow

Permanent docs describe current accepted truth.

Active plans describe current work.

Completed plans move to:

```text
docs/plans/completed/
```

Do not turn `PRODUCT.md` into a task checklist.

When code and documentation disagree, resolve the disagreement rather than leaving both.

---

## 25. Development quality checklist

Before marking a task complete:

- [ ] Scope matches the active plan.
- [ ] Shared logic is not duplicated between hosts.
- [ ] Platform dependencies remain isolated.
- [ ] Determinism is preserved.
- [ ] Tests cover meaningful behavior.
- [ ] Native and web implications are handled.
- [ ] Terminal cleanup was not weakened.
- [ ] Accessibility impact was considered.
- [ ] No native outbound or browser nonessential network activity was added.
- [ ] Content matches canon.
- [ ] Documentation is updated where needed.
- [ ] Full or appropriate checks pass.
- [ ] Diff contains no unrelated changes.

---

## 26. Recommended initial scripts

### `scripts/check.sh`

Should be readable shell code and stop on first failure.

### `scripts/dev-web.sh`

Build/watch Wasm and run website development server.

### `scripts/release.sh`

Optional helper that verifies version, clean tree, and checks before tagging. It should not hide release behavior.

### `scripts/validate-assets.sh`

Optional check for required assets and licenses.

Keep scripts cross-platform where reasonable, but do not create a complex scripting framework.

---

## 27. Common anti-patterns

Avoid:

- one giant application crate;
- games importing host libraries;
- platform-specific game rules;
- wall-clock-driven gameplay;
- arbitrary `sleep` inside simulations;
- `target-cpu=native` official binaries;
- full-screen terminal clears through manual ANSI each frame;
- unbounded logs;
- JSON blobs with no version;
- storing entire runtime game objects;
- generic “manager” modules with unclear ownership;
- premature ECS;
- premature async runtime;
- dozens of tiny crates;
- generated lore filler;
- release CI that nobody can explain.

---

## 28. Development definition of success

The process is working when:

- a coding agent can understand the task from the repository alone;
- implementation plans stay bounded;
- native and browser changes land together;
- checks are fast enough to run routinely;
- releases are simple enough to perform confidently;
- the code remains understandable after AI-assisted implementation;
- product and canon documents remain consistent with the application.
