# Plan 001 — First Signal

**Status:** Proposed  
**Target milestone:** Milestone 0 — First Signal  
**Owner:** Drilon Reçica  
**Primary game:** Signal Stack  
**Hosts:** Native terminal and browser

Related documents:

- `AGENTS.md`
- `docs/PRODUCT.md`
- `docs/ARCHITECTURE.md`
- `docs/DESIGN.md`
- `docs/CANON.md`
- `docs/DEVELOPMENT.md`
- `docs/DECISIONS.md`

---

## 1. Goal

Deliver the first complete Raster Nights product loop in both hosts:

```text
privacy notice
→ cold or warm boot
→ AfterHours launcher
→ Signal Stack
→ pause
→ game over
→ three-character tag
→ persisted local score
→ launcher
→ safe shutdown
```

The milestone proves architecture, input, rendering, deterministic simulation, persistence, terminal safety, browser integration, and core product identity.

This is not a throwaway prototype. Code and interfaces should be suitable for continued development, while remaining small enough to revise before 0.1.

---

## 2. User-visible result

After completing this plan, a user can:

### Native

```bash
cargo run -p raster-terminal
```

Then:

- see the privacy notice;
- boot the DRX-90;
- navigate the launcher;
- open Signal Stack details;
- play Standard Transmission;
- pause and resume;
- reach game over;
- enter a tag;
- view a persisted score;
- return to launcher;
- shut down;
- return to a clean terminal.

### Browser

- open the local website;
- press `POWER ON DRX-90`;
- receive keyboard focus;
- complete the same product loop;
- lose focus and see the session pause;
- refresh and retain settings and scores.

The native and browser simulations produce identical authoritative results for the same seed and timed actions.

---

## 3. In scope

### Repository

- Cargo workspace
- terminal app crate
- browser app crate
- shared engine crate
- display crate
- games crate
- storage crate
- testkit crate
- website skeleton
- simple scripts

### Shared engine

- identifiers
- canonical 100×36 grid
- fixed 60 Hz simulation
- deterministic RNG
- normalized input
- held-key tracking
- top-level app state machine
- game lifecycle
- game result envelope
- score-tag state
- pause and resize suspension

### Native host

- CLI skeleton
- terminal size check
- raw mode
- alternate screen
- cursor management
- keyboard input
- native mouse support in launcher where practical
- restoration guard
- panic cleanup
- resize events
- synchronous frame loop
- local file storage

### Browser host

- Wasm initialization
- Ratzilla WebGL2
- Canvas fallback
- `requestAnimationFrame`
- keyboard
- mouse
- focus and visibility pause
- local browser storage
- website `POWER ON`
- display scaling
- muted audio placeholder/no-op

### DRX-90 shell

- first-run privacy notice
- cold boot
- warm boot
- minimal AfterHours launcher
- Signal Stack detail screen
- pause screen
- game-over diagnostics
- tag entry
- score view
- shutdown sequence
- minimal NUL hint, not full introduction

### Signal Stack

- 10×20 visible matrix
- 4 hidden spawn rows
- seven packet shapes
- shuffled seven-packet bag
- five previews
- hold once per placement
- move
- rotate clockwise
- rotate counterclockwise
- soft drop
- hard drop
- wall kicks
- lock delay
- channel clearing
- increasing transmission rate
- score
- game over
- deterministic state hash
- basic canonical rendering

### Persistence

- settings schema
- score schema
- system-state schema
- format versions
- native atomic writes
- browser local writes
- corruption fallback
- in-memory test storage

### Testing

- game unit tests
- rotation tests
- bag tests
- scoring tests
- app state tests
- input mapping tests
- deterministic golden run
- rendering snapshots
- storage tests
- Wasm smoke build
- content/glyph validation basics

### CI

- format
- Clippy
- tests
- workspace build
- Wasm build
- website build

---

## 4. Out of scope

- Loopback
- Packet Sweep
- Bureau 9
- Mnemonic Nullway
- Afterline 99
- complete command shell
- fictional filesystem
- full manuals
- full audio
- soundtrack
- control-remapping UI
- touch controls
- advanced CRT effects
- attract mode
- Signal Stack Burst Calibration
- Signal Stack Transmission Repair
- global leaderboard
- replay export
- screenshots
- hosted SSH
- Windows
- signing or notarization
- Homebrew release automation
- localization content
- final web font

Interfaces may leave room for later features, but do not implement speculative systems.

---

## 5. Assumptions

- Stable Rust is available.
- The browser target supports WebAssembly.
- Ratzilla can provide WebGL2 and Canvas backends suitable for the canonical grid.
- The final bundled web font can be selected later; use a verified temporary open-source font or system monospace during local development.
- The exact package names may be adjusted, but crate responsibilities remain.
- Signal Stack tuning is provisional during this milestone.
- The project is not yet constrained by backward-compatible public save formats, but formats must still be versioned.

---

## 6. Architectural constraints

- No platform dependency in `raster-games`.
- No game rules in host crates.
- Rendering does not mutate game state.
- Authoritative game state uses integers/ticks.
- The random generator is seeded and injected.
- Native and browser use the same `SignalStack` implementation.
- Browser effects cannot change cell geometry.
- Storage is accessed through repositories or traits.
- Terminal cleanup is handled by RAII.
- No Tokio.
- No network requests.
- No plugin system.
- No separate web game renderer.

---

## 7. Proposed workspace skeleton

```text
raster-nights/
├── Cargo.toml
├── apps/
│   ├── terminal/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── web/
│       ├── Cargo.toml
│       └── src/
├── crates/
│   ├── raster-engine/
│   ├── raster-display/
│   ├── raster-games/
│   ├── raster-storage/
│   └── raster-testkit/
├── website/
├── content/
├── assets/
├── scripts/
└── docs/
```

`raster-audio` may be introduced now as semantic no-op infrastructure or deferred until audio work. Prefer deferral unless boot/game events already need a stable abstraction.

---

## 8. Work breakdown

## Workstream A — Repository foundation

- [ ] Create Cargo workspace.
- [ ] Add workspace package metadata.
- [ ] Add shared lint settings where appropriate.
- [ ] Add `.gitignore`.
- [ ] Add application and shared crates.
- [ ] Add website skeleton.
- [ ] Add `scripts/check.sh`.
- [ ] Confirm native workspace build.
- [ ] Confirm Wasm target build.
- [ ] Confirm website build.

### Acceptance

- One command validates all initial components.
- No crate has accidental platform dependencies.

---

## Workstream B — Core identifiers and types

Add explicit newtypes/enums for:

- [ ] `GameId`
- [ ] `ModeId`
- [ ] `RulesRevision`
- [ ] `RunSeed`
- [ ] `SimulationTick`
- [ ] `SimulationStep`
- [ ] `GridPoint`
- [ ] `GridSize`
- [ ] `GridRect`
- [ ] `GameStatus`
- [ ] `GameResult`
- [ ] `ThreeCharacterTag`

### Requirements

- Validate tags centrally.
- Avoid stringly typed game and mode logic.
- Provide serialization where persisted.
- Keep real semantic version separate from fictional version.

---

## Workstream C — Display façade

- [ ] Define canonical 100×36 constants.
- [ ] Define `GameCell`, style, and semantic color concepts.
- [ ] Implement an in-memory buffer adapter.
- [ ] Add `put`, `text`, `fill_rect`, `border`, and clipping.
- [ ] Handle out-of-bounds draws safely.
- [ ] Validate single-cell glyph inventory.
- [ ] Add readable buffer snapshot format.
- [ ] Render one test grid.

### Acceptance

- Buffer output is host-independent.
- Snapshots show glyph and style changes.
- Invalid-width canonical glyphs are rejected or flagged.

---

## Workstream D — Fixed-step clock

- [ ] Define 60 Hz canonical simulation.
- [ ] Implement accumulator helper.
- [ ] Clamp large stalls.
- [ ] Support pause without catch-up.
- [ ] Expose ticks, not wall-clock duration, to game logic.
- [ ] Test exact step counts.

### Acceptance

- Given host elapsed durations, the clock produces deterministic step counts.
- Resume after pause does not burst many updates.

---

## Workstream E — Input normalization

- [ ] Define physical key representation.
- [ ] Define global app actions.
- [ ] Define Signal Stack actions.
- [ ] Implement pressed/held/released state.
- [ ] Implement engine repeat timing.
- [ ] Add arrow and HJKL defaults.
- [ ] Add text-entry context.
- [ ] Add `Esc` hierarchy.
- [ ] Add `Ctrl+C` interrupt behavior.
- [ ] Add focus and resize events.

### Acceptance

- Native and browser mappings feed the same semantic actions.
- Held movement does not depend on OS repeat.
- Text entry does not interpret HJKL as navigation.

---

## Workstream F — Top-level application state machine

Implement:

- [ ] `PrivacyNotice`
- [ ] `ColdBoot`
- [ ] `WarmBoot`
- [ ] `Launcher`
- [ ] `SoftwareDetails`
- [ ] `Loading`
- [ ] `Playing`
- [ ] `Paused`
- [ ] `GameOver`
- [ ] `TagEntry`
- [ ] `Scores`
- [ ] `ResizeSuspended`
- [ ] `Shutdown`
- [ ] `FatalError`

### Acceptance

- Every transition is explicit.
- Tests cover primary user path and invalid actions.
- Pause and resize freeze game updates.

---

## Workstream G — Boot and launcher

### Privacy notice

- [ ] First-run state.
- [ ] Local acknowledgement.
- [ ] Clear no-network wording.

### Cold boot

- [ ] RCW/DRX-90 card.
- [ ] memory and device diagnostics.
- [ ] real local date warning through host-provided clock snapshot.
- [ ] one subtle NUL hint.
- [ ] skip on input.

### Warm boot

- [ ] abbreviated checks.
- [ ] restore last selected item.
- [ ] skip on input.

### Launcher

- [ ] Featured Software category.
- [ ] Signal Stack entry.
- [ ] unavailable/future entries may be omitted rather than shown as placeholders.
- [ ] keyboard selection.
- [ ] pointer selection.
- [ ] software detail screen.
- [ ] System Control placeholder only if functional enough.

### Acceptance

- No placeholder menu opens into empty screens.
- Boot text fits 100×36.
- Any input skips correctly without triggering accidental launcher action.

---

## Workstream H — Signal Stack model

### Matrix

- [ ] 10×20 visible cells.
- [ ] 4 hidden rows.
- [ ] collision queries.
- [ ] lock operation.
- [ ] channel clearing.
- [ ] spawn failure.

### Packets

- [ ] seven packet geometries.
- [ ] four rotations where applicable.
- [ ] deterministic shuffled bag.
- [ ] five preview queue.
- [ ] hold state.
- [ ] spawn positions.
- [ ] wall-kick table.

### Timing

- [ ] gravity by transmission rate.
- [ ] soft drop.
- [ ] hard drop.
- [ ] lock delay.
- [ ] limited lock refresh.
- [ ] rate advance every 10 channels.

### Scoring

- [ ] placement/drop points.
- [ ] channel clears.
- [ ] signal chains.
- [ ] sustained transmissions.
- [ ] phase rotations where implemented.
- [ ] zero-state matrix.
- [ ] score overflow policy.

### Status

- [ ] Running
- [ ] Paused by app
- [ ] Saturated/GameOver

### Acceptance

- Model has no rendering or host dependencies.
- Same seed produces same sequence.
- Unit tests cover every packet near walls and floor.

---

## Workstream I — Signal Stack rendering

Design a 100×36 layout with:

- [ ] title/status strip.
- [ ] central matrix.
- [ ] held packet.
- [ ] five previews.
- [ ] score.
- [ ] transmission rate.
- [ ] cleared channels.
- [ ] diagnostic status.
- [ ] danger state.
- [ ] pause overlay.
- [ ] game-over diagnostic.

### Requirements

- Distinguish packets by color and pattern/symbol.
- Keep matrix readable in High Contrast.
- Avoid browser-only dependency.
- Effects are simple and fast.

### Snapshots

- [ ] empty board
- [ ] mid-game
- [ ] near saturation
- [ ] multi-clear
- [ ] paused
- [ ] game over
- [ ] tag entry

---

## Workstream J — Game session and results

- [ ] Create new run request with seed.
- [ ] Route normalized actions.
- [ ] Freeze on pause.
- [ ] Produce result envelope.
- [ ] Determine qualifying local score.
- [ ] Enter tag.
- [ ] Persist score.
- [ ] Return to launcher.
- [ ] Handle restart.

### Acceptance

- One complete loop works without process restart.
- Repeated restarts do not leak state.
- Score record contains rules revision and mode.

---

## Workstream K — Storage

### Schemas

- [ ] Settings v1
- [ ] Scores v1
- [ ] System State v1

### Native

- [ ] per-user directory.
- [ ] temporary write.
- [ ] rename.
- [ ] corrupt-file preservation.
- [ ] data-directory override for development.

### Browser

- [ ] local adapter.
- [ ] quota/serialization error.
- [ ] schema version.
- [ ] reset.

### Tests

- [ ] round trip.
- [ ] missing file.
- [ ] corrupt file.
- [ ] incompatible version.
- [ ] isolated score corruption.
- [ ] atomic-write behavior where testable.

---

## Workstream L — Native host

- [ ] CLI.
- [ ] terminal capability check.
- [ ] 100×36 check.
- [ ] restoration guard.
- [ ] raw mode.
- [ ] alternate screen.
- [ ] cursor.
- [ ] input polling.
- [ ] mouse capture.
- [ ] resize events.
- [ ] frame loop.
- [ ] panic hook.
- [ ] shutdown.

### Manual cases

- [ ] normal exit.
- [ ] first and second `Ctrl+C`.
- [ ] panic.
- [ ] resize.
- [ ] tmux.
- [ ] SSH if available.

### Acceptance

Shell is usable immediately after every tested exit path.

---

## Workstream M — Browser host

- [ ] Wasm entry.
- [ ] Ratzilla WebGL2.
- [ ] Canvas fallback.
- [ ] animation frame loop.
- [ ] power-on.
- [ ] keyboard focus.
- [ ] mouse coordinates to grid.
- [ ] focus loss pause.
- [ ] hidden-tab pause.
- [ ] explicit resume.
- [ ] storage.
- [ ] display scaling.
- [ ] unsupported message.

### Acceptance

- No automatic boot.
- No automatic sound.
- Full grid visible.
- Same golden run as native.

---

## Workstream N — Website skeleton

- [ ] Raster Nights hero.
- [ ] tagline.
- [ ] `POWER ON DRX-90`.
- [ ] terminal/SSH/tmux message.
- [ ] install placeholder clearly marked as development until release.
- [ ] game catalog section for Signal Stack.
- [ ] privacy statement.
- [ ] source link placeholder.
- [ ] machine mount.
- [ ] accessible normal page navigation.

### Acceptance

The site remains useful when the Wasm application fails to load.

---

## Workstream O — Testing and CI

### Tests

- [ ] engine state transitions.
- [ ] input.
- [ ] clock.
- [ ] display.
- [ ] Signal Stack.
- [ ] storage.
- [ ] golden run native.
- [ ] golden run Wasm.
- [ ] snapshots.

### CI

- [ ] `cargo fmt --check`
- [ ] Clippy
- [ ] tests
- [ ] workspace build
- [ ] Wasm build
- [ ] website build

### Acceptance

CI is understandable from one workflow file or a very small set of workflows.

---

## 9. Suggested implementation order

1. Repository foundation
2. Core identifiers
3. Display buffer
4. Clock
5. Input
6. Minimal native host
7. Minimal browser host
8. App state machine
9. Privacy and boot
10. Launcher
11. Signal Stack model
12. Signal Stack rendering
13. Game session result
14. Storage
15. Host-specific polish
16. Website
17. Tests and CI hardening
18. Manual QA
19. Documentation synchronization

Do not build the full boot before both hosts can render the test grid.

Do not build storage before the result and settings schemas are clear.

---

## 10. Test plan

### Unit tests

- packet geometry;
- rotations;
- wall kicks;
- bag;
- collision;
- clears;
- scoring;
- level progression;
- lock timing;
- tag validation;
- app transitions;
- input repeat;
- clock;
- storage parsing.

### Golden run

At least one deterministic run:

```text
seed: fixed
mode: Standard Transmission
actions: tick-tagged
expected: score, channels, status, state hash
```

Execute in ordinary Rust and Wasm.

### Rendering snapshots

Review all major screens.

### Manual native

- macOS;
- Fedora;
- tmux;
- SSH where available;
- resize;
- mouse;
- Ctrl+C;
- panic cleanup.

### Manual browser

- WebGL2;
- Canvas fallback;
- keyboard;
- mouse;
- focus;
- resize/scaling;
- refresh persistence;
- first-run privacy.

---

## 11. Performance targets

Not hard release blockers until measurements stabilize, but measure:

- Signal Stack update under 1 ms;
- shared render under 4 ms;
- browser frame target 60 FPS;
- native target 60 where terminal permits, minimum stable 30;
- first displayed website/machine response prompt;
- Wasm size recorded.

No optimization work without a measured issue.

---

## 12. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Ratzilla glyph or backend mismatch | Restrict glyph inventory; Canvas fallback; adapter boundary |
| Native/browser input divergence | Normalize actions and test mappings |
| Terminal not restored after panic | RAII guard and panic-path manual tests |
| Signal Stack rules become too large | Limit Milestone 0 to canonical mode |
| Boot polish delays engine | Render test grid in both hosts before full boot |
| Storage schema churn | Version from first write; no stability promise before 0.1 |
| Browser focus creates lost runs | Immediate pause and explicit resume |
| AI-generated architecture drift | Enforce `AGENTS.md`, small commits, decision log |
| Web site framework consumes time | Keep Astro site static and small |
| Lore expands uncontrollably | Only required launch content in current plan |
| Unicode differs across terminals | Canonical tested single-cell glyph set |
| CI becomes slow | One primary Linux workflow and minimal build checks |

---

## 13. Acceptance criteria

### Product loop

- [ ] Privacy notice appears once and can be reopened later.
- [ ] Cold boot and warm boot work.
- [ ] Input skips boot without accidental double action.
- [ ] Launcher selects Signal Stack.
- [ ] Detail screen starts the game.
- [ ] Standard Transmission is playable.
- [ ] Pause/resume works.
- [ ] Game over produces a result.
- [ ] Qualifying score opens tag entry.
- [ ] Score persists.
- [ ] User returns to launcher.
- [ ] Shutdown restores terminal.

### Parity

- [ ] Native and browser use same game model.
- [ ] Same seed/actions produce same authoritative hash.
- [ ] Score matches across hosts.
- [ ] Rendering uses same canonical cell composition.

### Safety

- [ ] No network requests.
- [ ] No analytics.
- [ ] Terminal cleanup works.
- [ ] Resize freezes state.
- [ ] Browser focus loss freezes state.
- [ ] Corrupt score file does not erase settings.

### Quality

- [ ] No placeholder public screens.
- [ ] Text fits 100×36.
- [ ] High Contrast is readable.
- [ ] Critical states do not rely only on color.
- [ ] Checks pass.
- [ ] Documentation matches implementation.

---

## 14. Deferred follow-up

After First Signal:

- Loopback;
- hidden Packet Sweep;
- full 0.1 website content;
- Homebrew tap;
- official release archives;
- expanded settings;
- control remapping UI;
- touch;
- audio;
- manuals;
- deeper NUL behavior;
- Signal Stack secondary modes.

Do not start deferred work before the primary acceptance criteria are satisfied unless it unblocks the architecture.

---

## 15. Completion record

Fill when complete:

```text
Final commit:
Completion date:
Checks run:
Native platforms tested:
Browsers tested:
Known limitations:
Decision changes:
Follow-up plan:
```
