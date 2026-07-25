# Raster Nights Design System

**Status:** Working visual and interaction specification  
**Audience:** Designers, coding agents, UI implementers, content writers, and reviewers

This document defines how Raster Nights should look, move, sound, communicate, and respond. It covers the DRX-90 machine UI, game presentation, website, browser effects, accessibility, copy tone, and interaction conventions.

The design must support both native terminals and the browser. Every important screen must remain coherent as a 100×36 character-cell composition before browser-only effects are added.

---

## 1. Design vision

Raster Nights should feel like:

- an original 1990s home computer assembled from several plausible influences;
- a software archive discovered after its certified operating period;
- a machine designed for work that became unusually good at games;
- something colorful enough to evoke VGA-era software;
- something restrained enough to remain readable over SSH;
- a late-night computing environment with dry humor and occasional melancholy.

It must not feel like:

- a generic hacker dashboard;
- a modern web app with a monospace font;
- an exact DOS, Amiga, BIOS, or Windows replica;
- a green-text-only cyberpunk cliché;
- an endless stream of memes and fake errors;
- a pixel-art console pretending to be a terminal;
- a collection of unrelated game jams.

---

## 2. Design principles

### 2.1 Authentic structure, fictional details

Screen hierarchy, keyboard navigation, system diagnostics, file listings, manuals, and loading behavior should feel plausible. Device names, system capabilities, and diagnostics may be fictional and facetious.

### 2.2 Readability before atmosphere

Critical gameplay state must remain clear:

- without CRT effects;
- without audio;
- in a 256-color terminal;
- with a user-selected high-contrast palette;
- during moderate SSH latency.

### 2.3 One machine, multiple studios

R/OS provides a shared visual grammar. Individual games may vary in palette, title typography, effects, and framing as if produced by different fictional studios for the same hardware.

### 2.4 Effects communicate state

Animation, flashes, sound, and distortion should reinforce:

- confirmation;
- warning;
- urgency;
- score;
- failure;
- transition.

Effects should not exist merely to make every frame busy.

### 2.5 Ceremony is skippable

Boot, publisher cards, loading, game-over diagnostics, and shutdown are part of the identity, but input skips or accelerates them.

### 2.6 Mouse and touch are first-class but period-styled

The product should be approachable in the browser. Pointer interactions must look like they belong to the DRX-90 rather than a modern website layered on top.

---

## 3. Canonical display

### Logical dimensions

```text
Width:  100 cells
Height: 36 cells
```

All machine screens are designed at this size.

### Minimum native size

Native terminal requires 100×36. Larger terminals center the display or use controlled surrounding margin; they do not arbitrarily stretch internal layout.

### Browser scaling

The browser:

- preserves the entire grid;
- preserves cell aspect ratio;
- scales uniformly;
- uses integer scale where practical;
- may letterbox around the display;
- never crops gameplay to increase text size.

Portrait layouts keep the website and system screens usable. Real-time gameplay
uses landscape orientation and presents a rotate-device prompt when the
available portrait viewport cannot provide readable cells and controls.

### Safe areas

Recommended layout zones:

```text
Row 0             top system/title border
Rows 1–33         primary content
Row 34            status/help line
Row 35            bottom border or host status
```

Individual screens may vary, but global shortcuts and critical status should not jump unpredictably.

---

## 4. Cell and spacing tokens

### Standard spacing

- Outer display margin inside border: 1 cell
- Dialog horizontal padding: 2 cells
- Dialog vertical padding: 1 row
- Section gap: 1 row
- List marker width: 2–3 cells
- Key hint spacing: at least 2 cells between actions
- Minimum clickable pointer target: approximately 3×1 cells, larger where possible
- Touch targets exist outside or over the display and follow mobile accessibility sizing

### Borders

- Standard panels: single-line box drawing
- Critical or destructive confirmation: double-line border
- Selected game detail: single-line with emphasized title segment
- Subtle grouping: no border, use spacing and muted heading
- Avoid nesting more than two visible borders

### Alignment

- System labels: left aligned
- Numeric diagnostics: right aligned where useful
- Dates: `DD.MM.YYYY`
- Scores: grouped or padded consistently
- Menu shortcuts: aligned in a dedicated column
- Do not center long paragraphs in terminal cells

---

## 5. Typography

### R/OS interface

Use uppercase for:

- system headings;
- diagnostics;
- menu labels;
- status lines;
- commands;
- game-over messages.

Use sentence case for:

- manuals;
- website prose;
- explanatory help;
- longer error descriptions.

Avoid making every paragraph uppercase.

### Web font requirements

The bundled font should be:

- open-source and redistributable;
- truly monospaced;
- readable at small sizes;
- strong in box-drawing and block glyphs;
- visually period-appropriate without sacrificing clarity;
- distinct for `0/O`, `1/I/l`, and common punctuation;
- predictable for the selected Unicode subset.

Do not commit or distribute a font without confirming its license.

### Native terminal

Respect the user’s font. Do not attempt to install or replace it.

Provide `display-test` to diagnose:

- cell width;
- block glyphs;
- box drawing;
- colors;
- ambiguous glyphs.

### Game logos

Each flagship has a unique terminal-cell title treatment:

- Signal Stack: industrial diagnostic stencil
- Bureau 9: bureaucratic database typography
- Mnemonic Nullway: fragmented archival lettering
- Afterline 99: slanted vector racing mark

Bonus games use simpler standardized cover-disk title cards.

---

## 6. Color system

Colors are semantic, not merely decorative.

### Shared semantic roles

- Background
- Panel background
- Primary text
- Muted text
- System accent
- Selection
- Success
- Warning
- Critical
- Disabled
- Focus
- Player
- Rival
- Collectible
- Obstacle
- Grid
- Highlight

### Default theme: RCW Standard

Conceptual character:

- deep charcoal or midnight-blue background;
- warm off-white primary text;
- cyan system accents;
- amber secondary information;
- restrained red warnings;
- saturated VGA colors inside games.

Exact values should live in machine-readable theme data, not only here.

### Authored themes

#### RCW Standard

Balanced default for general use.

#### Amber Office

Warm monochrome shell with limited game color retention.

#### Green Phosphor

Classic terminal-inspired system shell. Games retain patterns and may reduce saturation.

#### Midnight VGA

Dark, colorful, higher-energy presentation.

#### High Contrast

Maximum readability and strong focus state.

#### Paper Terminal

Light background, particularly appropriate for Bureau 9 and manuals.

### Color rules

- Never encode a hazard only through red/green distinction.
- Selected state must include symbol, inverse, border, or modifier.
- Warning and critical states need text or pattern.
- Game pieces should have distinct patterns or symbols in addition to color.
- User themes override game recommendations where accessibility is concerned.

---

## 7. Glyph rules

Preferred glyph families:

- box drawing;
- full and half blocks;
- simple arrows;
- geometric symbols known to occupy one cell;
- ASCII fallback for diagnostics where needed.

Avoid:

- emoji presentation;
- ambiguous-width East Asian characters;
- complex combined graphemes;
- decorative symbols with inconsistent terminal support;
- glyphs whose visual meaning depends on a particular font.

Maintain a tested canonical glyph inventory.

Example inventory:

```text
█ ▀ ▄ ▌ ▐
░ ▒ ▓
─ │ ┌ ┐ └ ┘ ├ ┤ ┬ ┴ ┼
═ ║ ╔ ╗ ╚ ╝
▲ ▼ ◄ ► ◆ ● ■ □
```

Every glyph must be validated in supported native and browser renderers.

---

## 8. Motion system

### Timing categories

- Immediate input response: same frame or next simulation tick
- Menu selection pulse: 80–140 ms
- Panel transition: 120–220 ms
- Normal loading card: 0.5–1.5 seconds, skippable
- Game exit card: no more than 2 seconds, skippable
- Warm boot: 2–3 seconds, skippable
- Cold boot: 4–6 seconds, skippable
- Shutdown: 1–2 seconds, skippable

### Motion rules

- Do not animate every panel entrance.
- Keep cursor and selection feedback immediate.
- Avoid long easing that makes terminal UI feel sluggish.
- Browser post-processing must not delay gameplay input.
- Reduced Motion minimizes transitions, shake, rapid scrolling, and distortion.
- Important changes should remain understandable when animation is disabled.

---

## 9. CRT and browser effects

### Default profile: Authentic

Subtle:

- light scanlines;
- faint glow;
- slight vignette;
- no obvious curvature;
- no aggressive flicker;
- no heavy chromatic aberration over text.

### Profiles

#### Clean

No post-processing beyond necessary scaling.

#### Authentic

Subtle period-inspired display.

#### Intense

More visible glow, scanlines, and distortion, still playable.

#### Custom

Individual controls.

### Independently controllable

- screen shake;
- flashing;
- flicker;
- motion intensity;
- rapid palette changes.

### Prohibited defaults

- strong barrel distortion;
- text-blurring bloom;
- continuous random noise;
- constant horizontal roll;
- flicker likely to cause discomfort;
- effects that move cells away from collision coordinates.

---

## 10. Boot design

### Cold boot structure

1. Reçica Computer Works manufacturer card
2. DRX-90 model
3. R/OS ROM BIOS version
4. CPU and base memory
5. extended memory test
6. vector display initialization
7. audio array
8. archive/media detection
9. system clock warning if outside fictional period
10. unexplained resident process
11. launch AfterHours

Example:

```text
REÇICA COMPUTER WORKS
DRX-90 PERSONAL MULTIMEDIA SYSTEM

R/OS ROM BIOS 3.11

CPU TEST ............................. OK
BASE MEMORY ......................... 640K
EXTENDED MEMORY ..................... 16384K
VECTOR DISPLAY ADAPTER .............. READY
V/A-16 AUDIO ARRAY .................. READY
AFTERHOURS ENTERTAINMENT MODULE ..... FOUND
SYSTEM CLOCK ........................ OUTSIDE WARRANTY PERIOD
UNACCOUNTED RESIDENT PROCESS ........ 1
```

### Humor frequency

One or two unusual lines per boot maximum. Some boots may be entirely serious.

### Easter eggs

Rare diagnostics may reference:

- user patience;
- warranty expiration;
- unaccounted executables;
- NUL;
- unsupported current dates;
- fictional peripherals.

Do not fake access to real private files or user behavior beyond locally stored product state.

---

## 11. Privacy notice design

The first-run privacy notice is direct and not hidden in fiction. Its core
wording is shared, followed by an accurate host-specific network statement.

```text
┌════════════════════ LOCAL SYSTEM NOTICE ════════════════════┐
│                                                            │
│ Raster Nights has no accounts, analytics, telemetry or     │
│ advertising. It sends no scores or gameplay data.          │
│                                                            │
│ Settings, puzzle records and high scores remain on         │
│ this device.                                               │
│                                                            │
│                         [ ENTER ] CONTINUE                  │
└════════════════════════════════════════════════════════════┘
```

Native states that the installed application makes no outbound requests.
Browser states that browser play downloads the site and bundled application
files. Both variants must fit the canonical dialog.

R/OS framing is acceptable, but wording must remain unambiguous.

---

## 12. Launcher design

### Primary screen

```text
┌─ AFTERHOURS SOFTWARE ARCHIVE ────────────────────────────────────────────────┐
│ [F1] HELP  [F2] CATALOG  [F3] SCORES  [F10] R/OS                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  FEATURED SOFTWARE                                                          │
│                                                                             │
│  ► SIGNAL STACK            PUZZLE       21.11.1995                          │
│    BUREAU 9                LOGIC        08.02.1994                          │
│    MNEMONIC NULLWAY        ACTION       06.06.1997                          │
│    AFTERLINE 99            RACING       17.09.1998                          │
│                                                                             │
│  SHAREWARE ARCHIVE                                                          │
│    LOOPBACK                GRID          19.12.1993                          │
│    HAZARD REGISTRY         LOGIC         27.05.1994                          │
│    RELAY BREAKER           ARCADE        11.08.1996                          │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ ENTER Details    : Command    S Scores    ESC System                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Selection

Selected row uses at least two of:

- marker;
- inverse colors;
- bold modifier;
- focused border;
- pointer hover.

### Mouse

Rows are clickable. Hover is visible but subtle.

### Last selection

Warm boot restores selection. Cold boot returns to Featured Software and highlights the last played game.

---

## 13. Software detail screen

Required content:

```text
AFTERLINE 99
Nürnberg Night Systems · 17.09.1998

Race unauthorized signal routes before network sunrise.

MODE          NIGHT CHAMPIONSHIP
DIFFICULTY    STANDARD
LOCAL RECORD  184,220 — DRI

STEER         ← →
BRAKE         ↓
BOOST         SPACE
PAUSE         ESC

[ ENTER ] START     [ F1 ] MANUAL     [ ESC ] RETURN
```

Do not overload the screen with full lore. Detailed content belongs in the manual.

---

## 14. Software loading sequence

Loading is a fictional transition, not an actual progress indicator unless real loading occurs.

Possible lines:

```text
MOUNTING AH-0951.DSK ................. OK
CHECKING PROGRAM INDEX ............... OK
ALLOCATING 624K ...................... OK
LOADING SIGSTACK.EXE ................. ███████░░
```

Rules:

- short;
- skippable;
- different media styles by fictional year;
- do not lie about long operations;
- quiet mode minimizes it.

---

## 15. Command shell design

Prompt:

```text
R:\AFTERHRS>
```

Features:

- history;
- completion;
- simple filesystem;
- fixed-width output;
- readable error messages;
- no modern shell syntax beyond limited quoted arguments.

Error tone:

```text
R/OS: FILE NOT FOUND.
NUL: IT WAS HERE EARLIER.
```

Use NUL sparingly. Most errors should be plain.

The fictional filesystem should feel curated, not procedurally filled.

---

## 16. System Control design

Categories:

```text
DISPLAY
AUDIO
INPUT
ACCESSIBILITY
STARTUP
STORAGE
PRIVACY
DIAGNOSTICS
```

### Controls

- arrows/HJKL move;
- left/right change simple values;
- Enter opens detail;
- Esc backs out;
- mouse can select;
- focused element has clear marker.

### Destructive actions

Use double border and typed confirmation for `ERASE LOCAL ARCHIVE`.

---

## 17. Pause screen

```text
┌════════════════════ SESSION SUSPENDED ══════════════════════┐
│                                                            │
│                         RESUME                             │
│                         RESTART                            │
│                         CONTROLS                           │
│                         SETTINGS                           │
│                         RETURN TO AFTERHOURS               │
│                         SHUT DOWN                          │
│                                                            │
│ ESC resumes.                                               │
└════════════════════════════════════════════════════════════┘
```

The game remains visible but subdued where practical.

Pause must not create gameplay ambiguity when resumed.

---

## 18. Game-over and score-entry design

### Game over

Game-specific diagnostic sequence, brief and skippable.

Signal Stack example:

```text
SIGNAL CAPACITY EXCEEDED

CHANNEL MATRIX ................. SATURATED
PACKET INGRESS ................. FAILED
TRANSMISSION ................... TERMINATED
```

### Score entry

```text
┌════════════════════ NEW SYSTEM RECORD ══════════════════════┐
│                                                            │
│ SCORE                                              184,220 │
│                                                            │
│ ENTER OPERATOR IDENTIFICATION                              │
│                                                            │
│                          D R _                             │
│                                                            │
│ LEFT/RIGHT Select   UP/DOWN Change   ENTER Submit          │
└════════════════════════════════════════════════════════════┘
```

Remember the previous tag but require explicit submission.

---

## 19. NUL design

NUL means:

> **Nonessential Utility Layer**

### Visual form

Small cursor-like entity:

```text
[_]
```

or a minimal block/cursor manifestation.

### Personality

- helpful;
- dry;
- slightly insubordinate;
- knows more than R/OS admits;
- never chatty;
- not a modern assistant;
- not cute in a childish way;
- not threatening.

### Introduction

First session: only hints.

Later:

```text
R/OS: NONESSENTIAL UTILITY LAYER LOADED.
NUL: “NONESSENTIAL” WAS ADDED BY MANAGEMENT.
```

### Unofficial expansions

May appear as rumors or edits:

- Not Usually Listening
- Network Utility Lurker
- Nearly Useful Logic
- No User Left

Only Nonessential Utility Layer is canonical.

---

## 20. Copy and humor style

### Preferred

Dry, concise, institutional:

```text
SYSTEM CLOCK ........ OUTSIDE WARRANTY PERIOD
CONTINUING AT USER'S RISK.
```

```text
USER PATIENCE ....... UNVERIFIED
CONTINUING WITHOUT IT.
```

### Avoid

- meme references;
- emoji;
- internet slang;
- excessive fourth-wall jokes;
- long comedy routines;
- fake threats;
- fake data theft;
- fake claims that the machine read private files;
- constant NUL commentary.

### Error hierarchy

Serious user-impacting errors should lead with clear ordinary information. A joke may follow but never obscure the remedy.

---

## 21. Accessibility design

### Focus

Every interactive element has visible keyboard focus.

Do not rely on hover alone.

### Color

Use shape, label, pattern, or modifier in addition to color.

### Motion

Reduced Motion:

- disables shake;
- minimizes scrolling;
- removes flicker;
- shortens transitions;
- reduces corruption distortion;
- preserves timing and gameplay unless an assisted profile is selected.

### Flashing

No default effect should repeatedly flash large areas at unsafe frequencies.

### Audio

Every critical cue also has:

- text;
- HUD change;
- border pulse;
- icon;
- animation.

### Remapping

Controls screen shows actual current bindings, not hard-coded defaults.

### Mobile

Hardware controls must not cover critical HUD or game regions.

Real-time games use landscape controls. Portrait mode may show the website and
system interface, but must prompt for rotation before gameplay rather than crop
the grid.

### Semantic browser mirror

The Canvas/WebGL display has a synchronized, visually hidden semantic mirror for
system navigation, settings, manuals, dialogs, and Bureau 9. Focus in the
semantic mirror and focus shown in the cell UI refer to the same logical
element. Semantic actions re-enter the shared normalized action path; the mirror
does not implement parallel behavior.

---

## 22. Website design

### Overall style

A readable modern page styled like a high-end fictional 1990s product catalog.

### Hero

Required elements:

- Raster Nights logo;
- tagline;
- short explanation;
- `POWER ON DRX-90`;
- `INSTALL FOR TERMINAL`;
- SSH/tmux value proposition;
- sound toggle;
- clear focus behavior.

Example:

```text
RASTER NIGHTS                                      RCW CATALOG 99

LOST GAMES FOR A COMPUTER THAT NEVER EXISTED

[ POWER ON DRX-90 ]    [ INSTALL FOR TERMINAL ]

Play in the browser—or install it on your workstation,
development server, or home lab and launch it through
a normal terminal, tmux pane, or SSH session.
```

### Website content sections

- Play
- Game Catalog
- Terminal Installation
- How It Works
- DRX-90 Archive
- Manuals
- Source and Architecture
- Privacy
- Credits and Licenses

### Do not

- hide installation below many marketing sections;
- autoplay the machine;
- autoplay audio;
- use generic gradient-heavy SaaS styling;
- make all documentation part of the fictional shell;
- sacrifice ordinary semantic HTML.

---

## 23. Game visual identities

## 23.1 Signal Stack

### Mood

Industrial diagnostic system with colorful VGA signal packets.

### Board

- conventional rectangular matrix;
- extremely readable;
- effects never distort occupied cells;
- ghost/landing indicator optional and clearly distinct;
- side panels for hold, previews, rate, score, channels, and status.

### Effects

- normal clear: fast horizontal pulse;
- multi-clear: stronger sweep;
- maximum clear: brief freeze, diagnostic banner, optional shake;
- stack danger: border and meter warning, not permanent screen flicker.

## 23.2 Bureau 9

### Mood

Government logic terminal and archival database.

### Screen

- quiet;
- spacious;
- clear 3×3 sectors;
- selected cell, peers, conflicts, and candidates visually distinct;
- no active glitches while solving;
- calm theme options.

### Pointer

Mouse and touch selection should feel direct and precise.

## 23.3 Mnemonic Nullway

### Mood

Damaged memory landscape, strange but readable.

### Domains

- Document Cache
- Entertainment Memory
- Message Spool
- System Core
- Unindexed Space

### Corruption

May distort:

- edge labels;
- noncritical scenery;
- audio;
- color relationships.

Must not obscure immediate obstacle readability, especially in Reduced Effects mode.

### Player

REC-0 is a minimal bright geometric cursor-creature.

## 23.4 Afterline 99

### Mood

Nocturnal vector highway, unauthorized signal racing.

### Road

Pseudo-3D road projection uses:

- strong center and edge readability;
- clear curvature;
- depth-scaled rivals and traffic;
- readable route splits;
- restrained horizon detail.

### HUD

- speed;
- position;
- checkpoint time;
- signal integrity;
- boost heat;
- route warnings.

The display must not become a wall of gauges.

## 23.5 Loopback

Cover-disk utility feel. Fast startup. Simple network-grid visual language.

## 23.6 Hazard Registry

Inspection software. Calm deduction UI with restrained unstable-sector animation.

## 23.7 Relay Breaker

Bright arcade presentation with circuit-like block connections and responsive ball visibility.

## 23.8 Packet Sweep

Deliberately modest hidden developer game. Minimal presentation and a slightly unfinished but intentional character.

---

## 24. Fictional studios as design systems

Studios may have recurring title cards and sound identities.

- Frankenberg Logic Bureau: precise grids, measured typography, mathematical rhythm
- Nora Nova Interactive: experimental layouts, asymmetry, fragmented transitions
- Nürnberg Night Systems: slanted titles, nocturnal vector energy
- Noah Arc Labs: prototypes, unusual instrumentation, technical overlays
- Sara Circuitworks: polished commercial presentation
- Hofheim Home Software: warm, modest home-computer character
- Vranidoll Signal Works: signal pulses, diagnostic noise, foundational technology
- Prishtina Vector House: confident publisher cards, vivid vector identity
- Battenberg Byteforge: ambitious, slightly overengineered presentation
- Kosova Raster Union: collective/industry visual language when used

Studios must remain within the shared DRX-90 capability envelope.

---

## 25. Manuals and archive design

### In-machine manuals

- plain-text help;
- quick reference;
- searchable section index;
- keyboard-friendly;
- minimal illustration through cells.

### Website manuals

May resemble:

- boxed game manuals;
- registration cards;
- magazine reviews;
- hardware inserts;
- troubleshooting sheets;
- product catalogs.

### Lore density

A manual should prioritize:

1. how to start;
2. controls;
3. rules;
4. scoring;
5. accessibility;
6. fictional history.

Lore must not bury instructions.

---

## 26. Design review checklist

Before approving a screen:

- Does it fit 100×36?
- Does it work without browser effects?
- Is keyboard focus visible?
- Is the primary action obvious?
- Can the user skip ceremony?
- Is critical state distinguishable without color?
- Does text fit without truncating essential meaning?
- Are mouse targets clear?
- Does Reduced Motion preserve understanding?
- Does the tone match R/OS and the relevant studio?
- Is NUL used sparingly?
- Are names and dates canonical?
- Does the screen avoid resembling a generic modern dashboard?

---

## 27. Design definition of done

A visual feature is done when:

- canonical cell rendering is complete;
- native and browser show equivalent composition;
- focus and pointer states work;
- accessibility profiles work;
- no placeholder text remains;
- content matches canon;
- motion is skippable or reducible;
- snapshots cover stable layouts;
- browser-only polish is additive rather than required.
