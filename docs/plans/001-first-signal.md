# Plan 001 — First Signal

**Status:** Implementation complete — manual portability QA remains\
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

Implementation checkpoint (26.07.2026): the complete First Signal product loop
is implemented in shared code, composed in both hosts, and accepted in local
owner testing. Signal Stack rules, rendering, lifecycle, versioned persistence,
native atomic files, browser local storage, score entry, semantic screens,
browser power-on and shutdown, and native/Wasm deterministic golden runs are
complete. Portability-specific manual checks remain recorded below rather than
being inferred from general local testing.

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
- resize below 100×36 and explicitly resume after restoring the minimum size;
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
- explicitly resume after focus returns;
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

### Shared engine and display

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
- `wasm-pack --target web` package generation
- Ratzilla WebGL2
- Canvas fallback
- `requestAnimationFrame`
- keyboard
- mouse
- focus and visibility pause
- local browser storage
- website `POWER ON`
- display scaling
- semantic accessibility mirror for implemented system screens
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
- `wasm-pack --target web --no-pack` is the only Rust-to-browser packaging path;
  do not introduce Trunk alongside Astro.
- The final bundled web font can be selected later; use a verified temporary open-source font or system monospace during local development.
- Signal Stack tuning is provisional until Milestone 0 acceptance, but changes
  must update this plan and golden expectations. After persisted results exist,
  incompatible rule changes increment the rules revision.
- The project is not yet constrained by backward-compatible public save formats, but formats must still be versioned.

---

## 6. Architectural constraints

- No platform dependency in `raster-games`.
- No game rules in host crates.
- Rendering does not mutate game state.
- Authoritative game state uses integers/ticks.
- The run seed is injected. Games derive only documented deterministic
  substreams from it.
- Native and browser use the same `SignalStack` implementation.
- Browser effects cannot change cell geometry.
- Storage is accessed through repositories or traits.
- `raster-engine` does not depend on `raster-games`, `raster-storage`, or
  `raster-audio`; host applications compose and inject those implementations.
- Grid and cell geometry types belong to `raster-display`; game IDs, lifecycle,
  repository ports, and semantic UI types belong to `raster-engine`.
- Terminal cleanup is handled by RAII.
- No Tokio.
- Native makes no outbound requests. Browser requests are limited to bundled
  same-origin assets needed to load the site and Wasm application.
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

`raster-audio` is deferred. Milestone 0 uses the semantic audio sink port in
`raster-engine` with a no-op host sink and does not create the audio crate.

---

## 8. Work breakdown

## Workstream A — Repository foundation

- [x] Create Cargo workspace.
- [x] Add workspace package metadata.
- [x] Add shared lint settings where appropriate.
- [x] Add `.gitignore`.
- [x] Add application and shared crates.
- [x] Add `rand_chacha` and `rand_core` only to `raster-games`, with unused
      defaults disabled and MIT/Apache notices recorded.
- [x] Add website skeleton.
- [x] Add `scripts/check.sh`.
- [x] Configure ignored `wasm-pack` output for the website.
- [x] Confirm native workspace build.
- [x] Confirm `wasm-pack` browser build.
- [x] Confirm headless Wasm test execution.
- [x] Confirm website build.

### Acceptance

- One command validates all initial components.
- No crate has accidental platform dependencies.

---

## Workstream B — Core identifiers and types

Add explicit newtypes/enums for:

- [x] `GameId`
- [x] `ModeId`
- [x] `RulesRevision`
- [x] `RunSeed`
- [x] `SimulationTick`
- [x] `SimulationStep`
- [x] `GridPoint`
- [x] `GridSize`
- [x] `GridRect`
- [x] `GameStatus`
- [x] `GameResult`
- [x] `ThreeCharacterTag`
- [x] `InputCapability`
- [x] host-independent `SemanticUiTree`, `SemanticNode`, stable IDs, roles,
      states, and actions defined by `docs/ARCHITECTURE.md`
- [x] storage repository ports used by the application

### Requirements

- Validate tags centrally.
- Avoid stringly typed game and mode logic.
- Provide serialization where persisted.
- Keep real semantic version separate from fictional version.
- Keep persisted DTOs and migrations in `raster-storage`; do not make
  `raster-engine` depend on the storage implementation crate.

---

## Workstream C — Display façade

- [x] Define canonical 100×36 constants.
- [x] Define `GameCell`, style, and semantic color concepts.
- [x] Implement an in-memory buffer adapter.
- [x] Add `put`, `text`, `fill_rect`, `border`, and clipping.
- [x] Handle out-of-bounds draws safely.
- [x] Validate single-cell glyph inventory.
- [x] Add readable buffer snapshot format.
- [x] Render one test grid.

### Acceptance

- Buffer output is host-independent.
- Snapshots show glyph and style changes.
- Invalid-width canonical glyphs are rejected or flagged.

---

## Workstream D — Fixed-step clock

- [x] Define 60 Hz canonical simulation.
- [x] Implement accumulator helper.
- [x] Clamp large stalls.
- [x] Support pause without catch-up.
- [x] Expose ticks, not wall-clock duration, to game logic.
- [x] Test exact step counts.

### Acceptance

- Given host elapsed durations, the clock produces deterministic step counts.
- Resume after pause does not burst many updates.

---

## Workstream E — Input normalization

- [x] Define physical key representation.
- [x] Define global app actions.
- [x] Define Signal Stack actions.
- [x] Implement pressed/held/released state.
- [x] Implement engine repeat timing.
- [x] Support enhanced and compatibility input capabilities.
- [x] Add arrow and HJKL defaults.
- [x] Add text-entry context.
- [x] Add `Esc` hierarchy.
- [x] Add `Ctrl+C` interrupt behavior.
- [x] Add focus and resize events.

### Acceptance

- Native and browser mappings feed the same semantic actions.
- Held movement does not depend on OS repeat.
- Text entry does not interpret HJKL as navigation.
- Enhanced mode uses exact release events.
- Compatibility mode reports its limitations and expires held state without a
  release event.

### Compatibility hold contract

- The first raw press produces one immediate pressed action.
- A same-key press received within 60 simulation ticks arms a compatibility
  hold lease with an initial duration of 12 ticks.
- Each subsequent same-key press refreshes the lease to 12 ticks.
- Once armed, semantic repeats use the action's engine-defined repeat profile;
  raw repeat frequency never directly determines movement distance.
- Lease expiry produces a logical release.
- Fast repeated taps may be indistinguishable from a hold in compatibility mode;
  this limitation is accepted and disclosed.

---

## Workstream F — Top-level application state machine

Implement:

- [x] `PrivacyNotice`
- [x] `ColdBoot`
- [x] `WarmBoot`
- [x] `Launcher`
- [x] `SoftwareDetails`
- [x] `Loading`
- [x] `Playing`
- [x] `Paused`
- [x] `GameOver`
- [x] `TagEntry`
- [x] `Scores`
- [x] semantic tree for each implemented non-game state
- [x] `ResizeSuspended`
- [x] `Shutdown`
- [x] `FatalError`

### Acceptance

- Every transition is explicit.
- Tests cover primary user path and invalid actions.
- Pause and resize freeze game updates.

---

## Workstream G — Boot and launcher

### Privacy notice

- [x] First-run state.
- [x] Local acknowledgement.
- [x] Clear no-network wording.

### Cold boot

- [x] RCW/DRX-90 card.
- [x] memory and device diagnostics.
- [x] real local date warning through host-provided clock snapshot.
- [x] one subtle NUL hint.
- [x] skip on input.

### Warm boot

- [x] abbreviated checks.
- [x] restore last selected item.
- [x] skip on input.

### Launcher

- [x] Featured Software category.
- [x] Signal Stack entry.
- [x] unavailable/future entries may be omitted rather than shown as placeholders.
- [x] keyboard selection.
- [x] pointer selection.
- [x] software detail screen.
- [x] omit System Control until it has a functional screen.

### Acceptance

- No placeholder menu opens into empty screens.
- Boot text fits 100×36.
- Any input skips correctly without triggering accidental launcher action.

---

## Workstream H — Signal Stack model

### Matrix

- [x] 10×20 visible cells.
- [x] 4 hidden rows.
- [x] collision queries.
- [x] lock operation.
- [x] channel clearing.
- [x] spawn failure.

### Packets

- [x] seven packet geometries.
- [x] four rotations where applicable.
- [x] exact revision-1 spawn and rotation tables.
- [x] deterministic shuffled bag.
- [x] five preview queue.
- [x] hold state.
- [x] spawn positions.
- [x] wall-kick table.

### Timing

- [x] gravity by transmission rate.
- [x] soft drop.
- [x] hard drop.
- [x] lock delay.
- [x] limited lock refresh.
- [x] rate advance every 10 channels.

### Scoring

- [x] drop points.
- [x] channel clears.
- [x] signal chains.
- [x] sustained transmissions.
- [x] phase rotations.
- [x] zero-state matrix.
- [x] score overflow policy.

### Status

- [x] Running
- [x] Paused by app
- [x] Saturated/GameOver

### Acceptance

- Model has no rendering or host dependencies.
- Same seed produces same sequence.
- Unit tests cover every packet near walls and floor.

### Rules revision 1

Standard Transmission begins with `RulesRevision(1)`.

Coordinates use `x` increasing right and `y` increasing down. The matrix has
columns `0..=9`, hidden rows `0..=3`, and visible rows `4..=23`.

Spawn cells:

| Packet | Cells |
|---|---|
| I | `(3,2) (4,2) (5,2) (6,2)` |
| J | `(3,2) (3,3) (4,3) (5,3)` |
| L | `(5,2) (3,3) (4,3) (5,3)` |
| O | `(4,2) (5,2) (4,3) (5,3)` |
| S | `(4,2) (5,2) (3,3) (4,3)` |
| T | `(4,2) (3,3) (4,3) (5,3)` |
| Z | `(3,2) (4,2) (4,3) (5,3)` |

Orientation states are `0`, `R`, `2`, and `L`. JLSTZ use a 3×3 local box with
spawn origin `(3,2)` and pivot `(1,1)`. I uses a 4×4 local box with spawn origin
`(3,1)` and pivot `(1.5,1.5)`. Generate clockwise local geometry with
`(dx,dy) -> (-dy,dx)` around the pivot; counterclockwise is the inverse. Store
the resulting cells as explicit constant tables. O rotation is accepted as a
no-op and does not reset lock delay.

Kick candidates are tested in listed order using this document's positive-down
coordinates:

| JLSTZ transition | Candidate offsets |
|---|---|
| `0 -> R` | `(0,0) (-1,0) (-1,-1) (0,2) (-1,2)` |
| `R -> 0` | `(0,0) (1,0) (1,1) (0,-2) (1,-2)` |
| `R -> 2` | `(0,0) (1,0) (1,1) (0,-2) (1,-2)` |
| `2 -> R` | `(0,0) (-1,0) (-1,-1) (0,2) (-1,2)` |
| `2 -> L` | `(0,0) (1,0) (1,-1) (0,2) (1,2)` |
| `L -> 2` | `(0,0) (-1,0) (-1,1) (0,-2) (-1,-2)` |
| `L -> 0` | `(0,0) (-1,0) (-1,1) (0,-2) (-1,-2)` |
| `0 -> L` | `(0,0) (1,0) (1,-1) (0,2) (1,2)` |

| I transition | Candidate offsets |
|---|---|
| `0 -> R` | `(0,0) (-2,0) (1,0) (-2,1) (1,-2)` |
| `R -> 0` | `(0,0) (2,0) (-1,0) (2,-1) (-1,2)` |
| `R -> 2` | `(0,0) (-1,0) (2,0) (-1,-2) (2,1)` |
| `2 -> R` | `(0,0) (1,0) (-2,0) (1,2) (-2,-1)` |
| `2 -> L` | `(0,0) (2,0) (-1,0) (2,-1) (-1,2)` |
| `L -> 2` | `(0,0) (-2,0) (1,0) (-2,1) (1,-2)` |
| `L -> 0` | `(0,0) (1,0) (-2,0) (1,2) (-2,-1)` |
| `0 -> L` | `(0,0) (-1,0) (2,0) (-1,-2) (2,1)` |

Do not derive kick candidates through opaque arithmetic.

Run behavior:

- Fill the preview queue from a Fisher–Yates shuffled seven-packet bag.
- A first hold stores the active packet and consumes the next preview. A swap
  respawns the held packet in its spawn orientation and position.
- Hold becomes available again only after the active packet locks.
- Rate starts at 1 and advances after every ten cleared channels.
- Score the current clear using the rate at the start of lock resolution, then
  advance the rate if the new cleared-channel total crosses a ten-channel
  boundary.
- Gravity intervals, in ticks per row for rates 1–15, are
  `48, 43, 38, 33, 28, 23, 18, 13, 8, 6, 5, 4, 3, 2, 1`; later rates remain at
  one row per tick.
- Soft drop attempts one row per tick. Hard drop moves to the lowest legal row
  and locks immediately.
- Lock delay is 30 ticks at rates 1–5, 24 at 6–10, 18 at 11–14, and 12 at 15
  and above.
- A packet receives at most 15 successful grounded movement or rotation resets.
  Moving off the ground pauses the active lock timer but does not restore the
  reset allowance.
- After a lock and channel clear, any occupied hidden-row cell ends the run.
  Failure to spawn a packet also ends the run.
- The next packet spawns on the simulation tick after lock/clear resolution.

Signal Stack repeat defaults are a 10-tick delayed auto-shift and a two-tick
horizontal repeat interval.

Scoring uses `u64` saturating arithmetic. Multiply clear values by the current
rate:

| Event | Base points |
|---|---:|
| One channel | 100 |
| Two channels | 300 |
| Three channels | 500 |
| Four channels | 800 |
| Phase rotation, no clear | 400 |
| Phase rotation, one channel | 800 |
| Phase rotation, two channels | 1200 |
| Phase rotation, three channels | 1600 |

- Soft drop adds one point per descended row; hard drop adds two.
- A phase rotation requires the T packet, the last successful lateral/rotation
  maneuver before lock to be a rotation, and at least three occupied or
  out-of-bounds pivot-corner cells at lock. Gravity, soft drop, and hard-drop
  translation do not clear the last-maneuver marker; successful lateral movement
  does.
- Four-channel clears and channel-clearing phase rotations are sustained
  transmissions. Consecutive qualifying events multiply the later event's base
  clear points by `3/2` using integer arithmetic. A nonqualifying channel clear
  ends the sustained sequence; a lock with no clear leaves it unchanged.
- Consecutive locks that clear at least one channel form a signal chain. The
  first clear has chain index zero; later clears add
  `50 × chain index × rate`. A lock with no clear resets the chain.
- An empty matrix after clear resolution adds `2000 × rate`.

Use `rand_chacha::ChaCha8Rng` for bag shuffling. Derive each bag independently
from `(run seed, bag ordinal)` so authoritative state needs only the ordinal,
current bag, and cursor rather than opaque RNG internals.

Revision-1 seed derivation:

1. Write `run_seed` and `bag_ordinal` as the first two little-endian `u64`
   values of the 32-byte seed.
2. Initialize `state = run_seed ^ bag_ordinal.rotate_left(32) ^
   0x5349_4753_5441_434B` (`SIGSTACK`).
3. For each of the two remaining output lanes:
   - add `0x9E37_79B9_7F4A_7C15` to `state` with wrapping arithmetic;
   - set `z = state`;
   - set `z = (z ^ (z >> 30)) * 0xBF58_476D_1CE4_E5B9`;
   - set `z = (z ^ (z >> 27)) * 0x94D0_49BB_1331_11EB`;
   - set `z = z ^ (z >> 31)`;
   - append `z` in little-endian order.

All additions and multiplications wrap at 64 bits. Shuffle the canonical packet
order `I,J,L,O,S,T,Z` with Fisher–Yates from index 6 down to 1. For bound
`i + 1`, compute `range = 1u64 << 32` and
`limit = range - (range % bound)`. Draw `u32` values directly through
`rand_core::Rng::next_u32`,
promote to `u64`, reject values greater than or equal to `limit`, and use
`value % bound` as the swap index. Do not use a distribution helper whose
sampling algorithm may change between dependency versions. The RNG algorithm,
derivation, bounded sampling, and canonical order are part of the rules
revision.

State hashes use an explicit canonical little-endian field encoding and a
project-owned FNV-1a 64-bit implementation with offset basis
`14695981039346656037` and prime `1099511628211`.

Encode, in order: rules revision; status; run seed; 240 row-major matrix cells;
active packet/rotation/position; hold packet and availability; preview queue;
current bag, cursor, and bag ordinal; score; cleared channels; rate; gravity
counter; lock timer and reset count; last authoritative action; signal-chain
index; sustained-transmission state; and pending spawn state. Use explicit enum
tags, option-presence bytes, fixed-width integers, and length prefixes where
needed. Never hash memory layouts, derive output from map iteration order, or
use Rust's default hasher. The hash is a regression identifier, not a security
primitive.

---

## Workstream I — Signal Stack rendering

Design a 100×36 layout with:

- [x] title/status strip.
- [x] central matrix.
- [x] held packet.
- [x] five previews.
- [x] score.
- [x] transmission rate.
- [x] cleared channels.
- [x] diagnostic status.
- [x] danger state.
- [x] pause overlay.
- [x] game-over diagnostic.

### Requirements

- Distinguish packets by color and pattern/symbol.
- Keep matrix readable in High Contrast.
- Avoid browser-only dependency.
- Effects are simple and fast.

### Snapshots

- [x] full empty-board structured golden snapshot
- [x] full mid-game structured golden snapshot
- [x] full near-saturation structured golden snapshot
- [x] full pending multi-clear structured golden snapshot
- [x] full paused structured golden snapshot
- [x] full game-over structured golden snapshot
- [x] full tag-entry structured golden snapshot

---

## Workstream J — Game session and results

- [x] Create new run request with seed.
- [x] Route normalized actions.
- [x] Freeze on pause.
- [x] Produce result envelope.
- [x] Determine qualifying local score.
- [x] Enter tag.
- [x] Persist score.
- [x] Return to launcher.
- [x] Handle restart.

### Acceptance

- One complete loop works without process restart.
- Repeated restarts do not leak state.
- Score record contains rules revision and mode.
- Standard Transmission keeps ten records per rules revision.
- Equal cutoff scores do not displace the older record.

---

## Workstream K — Storage

### Schemas

- [x] Settings v1
- [x] Scores v1
- [x] System State v1

### Native

- [x] per-user directory.
- [x] temporary write.
- [x] rename.
- [x] corrupt-file preservation.
- [x] data-directory override for development.
- [x] in-memory fallback with visible persistence warning.

### Browser

- [x] local adapter.
- [x] quota/serialization error.
- [x] schema version.
- [x] reset.
- [x] in-memory fallback with visible persistence warning.

### Tests

- [x] round trip.
- [x] missing file.
- [x] corrupt file.
- [x] incompatible version.
- [x] isolated score corruption.
- [x] atomic-write behavior where testable.

---

## Workstream L — Native host

- [x] CLI.
- [x] terminal capability check.
- [x] 100×36 check.
- [x] restoration guard.
- [x] raw mode.
- [x] alternate screen.
- [x] cursor.
- [x] input polling.
- [x] mouse capture.
- [x] resize events.
- [x] frame loop.
- [x] panic hook.
- [x] enhanced keyboard capability detection.
- [x] keyboard enhancement restoration.
- [x] compatibility input mode.
- [x] centered-display pointer offset mapping.
- [x] shutdown.

### Manual cases

- [x] normal exit.
- [x] first and second `Ctrl+C`.
- [ ] panic.
- [ ] resize.
- [ ] explicit resume after returning to 100×36 or larger.
- [ ] tmux.
- [ ] SSH if available.

### Acceptance

Shell is usable immediately after every tested exit path.

---

## Workstream M — Browser host

- [x] Wasm entry.
- [x] Ratzilla WebGL2.
- [x] Canvas fallback.
- [x] animation frame loop.
- [x] power-on.
- [x] dynamic import only after `POWER ON`.
- [x] keyboard focus.
- [x] mouse coordinates to grid.
- [x] focus loss pause.
- [x] hidden-tab pause.
- [x] explicit resume.
- [x] storage.
- [x] display scaling.
- [x] unsupported message.
- [x] semantic mirror for privacy, launcher, details, pause, game over, and tag entry.

### Acceptance

- No automatic boot.
- No automatic sound.
- Full grid visible.
- Same golden run as native.
- Semantic and cell focus remain synchronized.

---

## Workstream N — Website skeleton

- [x] Raster Nights hero.
- [x] tagline.
- [x] `POWER ON DRX-90`.
- [x] terminal/SSH/tmux message.
- [x] explicit development-status installation notice with no fake command.
- [x] game catalog section for Signal Stack.
- [x] privacy statement.
- [x] source link to `https://github.com/drilonrecica/raster-nights`.
- [x] machine mount.
- [x] accessible normal page navigation.

### Acceptance

The site remains useful when the Wasm application fails to load.

---

## Workstream O — Testing and CI

### Tests

- [x] engine state transitions.
- [x] input.
- [x] clock.
- [x] display.
- [x] Signal Stack.
- [x] storage.
- [x] golden run native.
- [x] golden run Wasm.
- [x] enhanced and compatibility input.
- [x] semantic tree and browser mirror.
- [x] full structured rendering golden snapshots.

### CI

- [x] `cargo fmt --check`
- [x] Clippy
- [x] tests
- [x] workspace build
- [x] `wasm-pack` build
- [x] headless Wasm smoke tests
- [x] website build

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
16. Semantic browser mirror
17. Website
18. Tests and CI hardening
19. Manual QA
20. Documentation synchronization

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
- enhanced and compatibility input;
- semantic tree generation;
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
| Legacy terminals lack release events | Capability tiers, compatibility hold lease, and visible diagnostics |
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

- [x] Privacy notice appears once and can be reopened later.
- [x] Cold boot and warm boot work.
- [x] Input skips boot without accidental double action.
- [x] Launcher selects Signal Stack.
- [x] Detail screen starts the game.
- [x] Standard Transmission is playable.
- [x] Pause/resume works.
- [x] Game over produces a result.
- [x] Qualifying score opens tag entry.
- [x] Score persists.
- [x] User returns to launcher.
- [x] Shutdown restores terminal.

### Parity

- [x] Native and browser use same game model.
- [x] Same seed/actions produce same authoritative hash.
- [x] Score matches across hosts.
- [x] Rendering uses same canonical cell composition.

### Safety

- [x] Native makes no outbound requests.
- [x] Browser makes no requests beyond bundled same-origin application assets.
- [x] No analytics.
- [x] Terminal cleanup works.
- [x] Resize freezes state.
- [x] Resize recovery requires explicit resume.
- [x] Browser focus loss freezes state.
- [x] Corrupt score file does not erase settings.

### Quality

- [x] No placeholder public screens.
- [x] Text fits 100×36.
- [ ] High Contrast is readable.
- [x] Critical states do not rely only on color.
- [x] Checks pass.
- [x] Documentation matches implementation.

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
