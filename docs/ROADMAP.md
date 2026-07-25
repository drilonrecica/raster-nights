# Raster Nights Roadmap

**Status:** High-level milestone roadmap  
**Scheduling:** No fixed dates  
**Release rule:** Ship when quality gates are satisfied

This roadmap communicates sequence and scope without promising calendar dates. It should remain high-level. Detailed tasks belong in active files under `docs/plans/`.

---

## 1. Roadmap principles

- Native and browser progress together.
- Tagged releases are polished enough for ordinary users.
- The first milestone proves the complete product loop.
- The racer and runner are delayed until the shared system is stable.
- No backend, accounts, analytics, or plugin work enters v1.
- Scope is reduced by cutting optional effects and secondary content before cutting core parity or safety.
- New games are added only when the previous milestone is stable.

---

## 2. Milestone 0 — First Signal

**Status:** Implementation complete; release-platform verification remains

### Purpose

Prove the shared architecture before public 0.1 scope is complete.

### Required outcome

A user can:

```text
power on
→ see privacy notice
→ boot DRX-90
→ open launcher
→ start Signal Stack Standard Transmission
→ play
→ pause
→ lose
→ enter a tag
→ save a score
→ return to launcher
→ shut down safely
```

This works in native terminal and browser.

### Deliverables

- Cargo workspace;
- terminal app;
- browser app;
- 100×36 display;
- normalized input;
- deterministic 60 Hz simulation;
- native restoration guard;
- Ratzilla WebGL2 and Canvas fallback;
- top-level application state machine;
- privacy notice;
- cold and warm boot;
- minimal launcher;
- Signal Stack core;
- local score storage;
- rendering snapshots;
- golden runs;
- simple CI.

### Exit criteria

- identical authoritative run hashes across native and Wasm;
- safe normal and panic terminal cleanup;
- resize suspension;
- browser focus suspension;
- no native outbound requests or browser requests beyond bundled same-origin
  application assets;
- all checks pass.

Detailed plan: `docs/plans/001-first-signal.md`

---

## 3. Raster Nights 0.1 — System Preview

### Purpose

Release a complete small product, not a framework preview.

### Games

- Signal Stack — polished Standard Transmission
- Loopback — polished Quick Circuit
- Packet Sweep — hidden miniature game

### System

- public website;
- `POWER ON DRX-90`;
- cold and warm boot;
- AfterHours launcher;
- software details;
- local settings;
- scoreboards;
- tag entry;
- mouse and keyboard;
- basic landscape touch;
- substantial manuals and canon;
- privacy notice;
- diagnostics;
- direct launch and quiet mode.

### Platforms

- macOS Apple Silicon
- macOS Intel
- Fedora-compatible Linux x86-64
- browser

### Distribution

- Homebrew tap
- Cargo
- release archives
- website
- checksums

### Exit criteria

- Signal Stack is enjoyable beyond novelty;
- Loopback serves short SSH/tmux sessions;
- official supported artifacts install;
- website and browser build work;
- no placeholder screens;
- no known terminal corruption;
- no hidden network activity;
- browser network behavior is limited to loading bundled same-origin assets;
- release licensing and identity notices are complete.

---

## 4. Raster Nights 0.2 — Logic Archive

### Purpose

Add the first turn-based flagship and validate richer pointer/touch interaction and puzzle persistence.

### Major deliverable

**Bureau 9**

- 240 curated puzzles;
- Case Archive;
- Assign Case;
- Paper, Assisted, Guided;
- pencil marks;
- undo/redo;
- explanatory hints;
- keyboard, mouse, touch;
- completion records.

### System improvements

- manual archive;
- improved settings;
- storage migrations;
- richer website manuals;
- accessibility refinement.

### Exit criteria

- puzzle catalog validated;
- all puzzles have unique solutions and stable IDs;
- hint system explains supported techniques correctly;
- touch experience is practical;
- active solving remains distraction-free.

---

## 5. Raster Nights 0.3 — Shareware Archive

### Purpose

Make AfterHours feel like a genuine software collection.

### Games

- Hazard Registry
- Relay Breaker

### System improvements

- shareware category;
- publisher and studio index;
- release timeline;
- more filesystem lore;
- attract screens;
- refined direct launch.

### Exit criteria

- bonus games are polished, not filler;
- each introduces one original twist;
- catalog browsing remains fast;
- fictional presentation is substantial but bounded.

---

## 6. Raster Nights 0.4 — Mnemonic Nullway

### Purpose

Prove pseudo-3D perspective gameplay, authored procedural assembly, corruption effects, and richer audio.

### Major deliverable

**Mnemonic Nullway**

- REC-0;
- lane movement;
- jump;
- slide;
- phase shift;
- corruption;
- five memory domains;
- finite Recovery Session;
- Continuous Recovery;
- authored section library;
- V-SCAPE 2.2.

### Architecture work

- fixed-point runner movement;
- section contracts;
- projection rendering;
- richer golden runs;
- reduced-effects mode for corruption;
- reactive audio layers.

### Exit criteria

- full finite run is fair and readable;
- phase mechanic produces meaningful route decisions;
- authored generation cannot assemble impossible transitions;
- browser remains smooth;
- native remains playable at target rate.

---

## 7. Raster Nights 0.5 — Afterline 99

### Purpose

Deliver the primary showcase flagship.

### Major deliverable

**Afterline 99**

- six signal craft;
- five routes;
- championship;
- time attack;
- single route;
- endless highway;
- rival and traffic simulation;
- drafting;
- drifting;
- boost heat;
- signal integrity;
- branching routes;
- multiple endings;
- V-SCAPE 3.7.

### Architecture work

- track-relative fixed-point physics;
- route topology;
- rival AI;
- collision;
- optimized pseudo-3D renderer;
- performance tooling;
- expanded audio.

### Exit criteria

- racing is mechanically engaging;
- route choices matter;
- keyboard steering feels analog and controllable;
- terminal presentation is convincing;
- performance budgets are met;
- reduced-effects mode remains readable.

---

## 8. Raster Nights 0.6 — Complete Archive Candidate

### Purpose

Integrate and stabilize the full catalog.

### Work

- all flagship and bonus manuals;
- catalog consistency;
- game-to-game references;
- accessibility audit;
- storage migration audit;
- platform QA;
- website archive;
- installation refinement;
- performance and binary-size review;
- documentation cleanup.

### Exit criteria

- no experimental placeholder systems;
- all games share expected lifecycle;
- all records migrate;
- all supported platforms install cleanly;
- canon is internally consistent.

---

## 9. Raster Nights 1.0 — Complete Archive

### Required catalog

Flagship:

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

### Required system quality

- native and browser parity;
- stable local persistence;
- substantial manuals and canon;
- accessibility baseline;
- deterministic tests;
- simple release process;
- no backend;
- no analytics;
- no plugin system;
- polished website;
- macOS and Fedora support at minimum.

### 1.0 does not require

- global leaderboard;
- Windows;
- hosted SSH;
- multiplayer;
- accounts;
- replay export;
- cloud saves;
- code signing;
- notarization.

---

## 10. Post-1.0 priorities

1. New substantial games
2. Stability and compatibility
3. Deeper cross-game lore
4. Audio and visual refinement
5. Better mobile experience
6. Optional online experiments only if clearly valuable

Potential future genres:

- card or strategy game;
- roguelike;
- platformer;
- shooter;
- rally navigation;
- simulation;
- rhythm game adapted carefully to terminal timing.

New games may occupy any fictional date from 05.10.1993 through 31.12.1999.

---

## 11. Features intentionally deferred

- Windows official support
- Linux ARM official artifacts
- musl builds
- RPM/COPR repository
- public global top-ten leaderboard
- controller support
- replay browser and export
- screenshot and recording export
- hosted SSH service
- shared machine-wide scores
- full screen-reader gameplay
- localization content
- custom web font creation
- trademark registration
- complex CI matrices

Deferred does not mean planned. Each requires a new decision.

---

## 12. Scope-cut rules

Protect:

- native/browser parity;
- terminal safety;
- core gameplay;
- deterministic simulation;
- privacy;
- accessibility fundamentals;
- coherent boot and launcher;
- local persistence.

Cut or simplify first:

1. soundtrack volume;
2. CRT sophistication;
3. advanced mobile racer/runner support;
4. shell depth;
5. lore volume;
6. remapping UI polish;
7. attract mode sophistication;
8. secondary modes;
9. peripheral bonus-game content.

---

## 13. Roadmap maintenance

Update this file only when milestone sequence or public scope changes.

Do not add:

- dates;
- weekly tasks;
- individual bug IDs;
- implementation checkboxes;
- speculative features with no decision.

Detailed execution belongs in `docs/plans/`.
