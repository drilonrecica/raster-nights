# Plan 002 — System Preview

**Status:** Active  
**Target milestone:** 0.1 system preview  
**Owner:** Drilon Reçica  
**Hosts:** Native terminal and browser

Related documents:

- `AGENTS.md`
- `docs/PRODUCT.md`
- `docs/ARCHITECTURE.md`
- `docs/DESIGN.md`
- `docs/CANON.md`
- `docs/DEVELOPMENT.md`
- `docs/DECISIONS.md`
- `docs/ROADMAP.md`

## Goal

Deliver a release-ready local `v0.1.0-rc.1` candidate with two advertised
games, the hidden Packet Sweep game, completed 0.1 system and authored content,
browser touch support, and deterministic release packaging.

The candidate is not a public release. Publishing, deployment, and the final
`v0.1.0` tag require explicit owner acceptance. Final macOS runtime acceptance
requires execution on supported macOS hardware.

## Execution order

1. Close locally testable First Signal QA and accessibility checks.
2. Stabilize catalog, startup, persistence, palette, and content interfaces.
3. Implement Loopback as a native/WebAssembly vertical slice.
4. Implement Packet Sweep and its Signal Stack discovery flow.
5. Complete settings, manuals, archive content, touch controls, and website.
6. Add packaging, tag-triggered CI, Homebrew generation, and release documents.
7. Run all available acceptance gates and create a local annotated
   `v0.1.0-rc.1` tag only from a clean worktree.

Plan 001 moves to `docs/plans/completed/` only after its remaining locally
testable checks pass. SSH QA is attempted only when a local SSH service already
exists; unavailable external environments are recorded honestly.

## Shared architecture and interfaces

- [x] Rename the terminal package and binary to expose `raster-nights` while
      retaining `apps/terminal/`.
- [x] Add a dependency-free CLI parser for normal launch, `--quiet`,
      `display-test`, `diagnostics [--output PATH]`, `validate-content`, and
      `play <signal-stack|loopback> [--quick] [--seed N]`.
- [x] Keep Packet Sweep unavailable through direct launch until local unlock.
- [x] Add `StartupOptions` and `DirectLaunchRequest` to the engine.
- [x] Never skip the privacy acknowledgement.
- [x] Make normal direct launch use shortened controls/loading and `--quick`
      skip only nonessential ceremony.
- [x] Make `--quiet` shorten animations without mutating saved settings.
- [x] Replace hard-coded catalog metadata with versioned bundled JSON.
- [x] Make `GameDescriptor` owned and typed, including fictional version,
      catalog number, controls, modes, and advertised/hidden visibility.
- [x] Expose advertised and hidden registrations separately.
- [x] Fail startup and `validate-content` clearly for invalid bundled content.
- [ ] Add stable game-result discovery markers.
- [x] Migrate system state v1 to v2 while preserving valid v1 files and
      existing privacy, selection, mode, and tag values.
- [x] Persist Packet Sweep trace-revealed and unlocked state.
- [x] Add R/OS Standard, Amber, Green, Midnight VGA, High Contrast, and Paper
      palette conversion in both hosts.
- [ ] Complete settings for palette, reduced motion, quiet operation, and
      browser CRT effects that actually exist.
- [ ] Add validated Wasm `touch_action(action, phase)` input using the same
      normalized action path as keyboard input.
- [ ] Add manual index/detail app states and semantic nodes.
- [ ] Make manuals and website pages consume the same versioned content.

## Loopback — Quick Circuit revision 1

- [ ] Implement a deterministic 24×20 logical arena rendered double-width.
- [ ] Run for 7,200 ticks with three integrity points and a four-segment route
      initially moving right.
- [ ] Reject direct reversal.
- [ ] Spawn one deterministic payload at a time.
- [ ] Implement two paired port sets that preserve heading.
- [ ] Increase the next-payload multiplier on port traversal to a maximum of
      four and reset it on collection.
- [ ] Score payloads at `100 × multiplier`.
- [ ] Award `500 × remaining integrity` on timed completion.
- [ ] Increase speed at 8, 16, 24, and 32 payloads from 12 to 6 ticks per move.
- [ ] On wall/self collision, remove integrity, reset multiplier and route, and
      grant 60 ticks of visible recovery protection while the timer continues.
- [ ] End on zero integrity; complete on timer expiry.
- [ ] Add pause/restart, records, controls, semantic status, deterministic hash,
      native/Wasm golden run, rule tests, and structured rendering snapshots.
- [ ] Keep Open Loop deferred.

## Packet Sweep revision 1

- [ ] Implement one fixed 24×18 logical arena.
- [ ] Run for 5,400 ticks with three integrity points.
- [ ] Collect one valid packet at a time with a four-way maintenance cursor.
- [ ] Start with three deterministic moving checksum errors.
- [ ] Add one error every 15 packets to a maximum of eight.
- [ ] Use seeded headings and deterministic wall reflection.
- [ ] Score valid packets at `100 + 25 × min(current streak, 20)`.
- [ ] On collision, remove integrity, reset streak, return to center, and grant
      60 ticks of visible protection.
- [ ] End on zero integrity; complete on timer expiry.
- [ ] Add its own registration, rules revision, records, deterministic golden
      run, and structured rendering snapshots.
- [ ] Keep Packet Sweep absent from ordinary catalog and website game listings.

## Discovery flow

- [ ] Record a trace discovery after a Signal Stack zero-state clear at
      transmission rate 5 or above.
- [ ] Reveal `TRACE90` in post-run diagnostics.
- [ ] Expose an accessible trace-entry action from Signal Stack details.
- [ ] Persist unlock and launch Packet Sweep after entering `TRACE90`.
- [ ] Replace trace entry with trace recall after unlock.

## System, content, and website

- [x] Complete locally available Fedora First Signal QA: panic restoration,
      resize/resume, tmux, and High Contrast.
- [x] Attempt local SSH QA only if an SSH service is already available. No
      local SSH service was active on 26.07.2026.
- [x] Add sanitized diagnostics without usernames, full paths, environment
      values, network activity, or automatic upload.
- [ ] Add versioned catalog, manual, and archive content for the DRX-90/R/OS,
      Reçica Computer Works, Signal Stack, Loopback cover-disk notes, existing
      canonical studios, two restrained reviews, curated filesystem entries,
      and several NUL interactions.
- [ ] Draft copy strictly from existing canon; add no companies, dates,
      personal details, or major lore.
- [ ] Expand the Astro site with manuals/archive pages, touch controls, current
      installation guidance, accessibility behavior, and accurate status.
- [ ] Keep audio, Open Loop, advanced remapping, elaborate CRT effects, and
      additional Signal Stack modes deferred.

## Release candidate

- [ ] Add deterministic packaging for Linux x86-64, macOS Intel, macOS Apple
      Silicon, browser bundle, and SHA-256 checksums.
- [ ] Include license and notice files in every archive.
- [ ] Add tag-triggered CI using `ubuntu-24.04`, `macos-15-intel`, and
      `macos-15`.
- [ ] Upload workflow artifacts without publishing a release or deploying.
- [ ] Add a Homebrew formula template and generator consuming archive URLs and
      SHA-256 values.
- [ ] Add `CHANGELOG.md`, release notes, asset/license manifests, trademark
      policy, and documentation licensing mappings.
- [ ] Create annotated local `v0.1.0-rc.1` only after every available gate
      passes and the worktree is clean.
- [ ] Leave final `v0.1.0` blocked on macOS runtime verification and explicit
      owner release acceptance.

## Acceptance

- [ ] Unit tests cover both new games, CLI parsing, content validation, state
      migration/recovery, palettes, and touch press/release behavior.
- [ ] Native and Firefox-headless Wasm golden runs pass for Signal Stack,
      Loopback, and Packet Sweep.
- [ ] Structured snapshots cover the specified game and system states.
- [ ] Host integration covers direct/quick/quiet launch, diagnostics, content
      validation, browser lifecycle/touch, and terminal cleanup/resize/tmux.
- [ ] `./scripts/check.sh` passes.
- [ ] Package extraction, binary smoke tests, website production build, archive
      manifests, and checksum verification pass.
- [ ] Browser runtime initiates no non-bundled requests after loading.
- [ ] Known unavailable external runtime checks are recorded and not marked
      complete.

## Deliberately deferred

- Audio
- Loopback Open Loop
- Advanced input remapping
- Elaborate CRT effects
- Additional Signal Stack modes
- Public release publication or deployment
- Final `v0.1.0` tag
