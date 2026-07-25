# Raster Nights Fictional Canon

**Status:** Canonical working reference  
**Audience:** Writers, designers, coding agents, game developers, and reviewers

This document defines the fictional world presented inside Raster Nights. It exists to keep names, dates, studios, technologies, tone, and cross-references consistent.

The fiction is inspired by personal names and places supplied by the creator, but it must not expose private biographical information. Use transformed company and product identities only. Do not invent family relationships, addresses, personal histories, schools, employers, or claims about real people.

---

## 1. Canon boundary

Raster Nights has two layers:

### Real-world layer

- Project: Raster Nights
- Creator and curator: Drilon Reçica
- Technology: Rust, terminal rendering, WebAssembly
- Source repository and public documentation

### Fictional layer

- Manufacturer: Reçica Computer Works
- Computer: DRX-90
- Operating system: R/OS
- Game archive: AfterHours
- Resident utility: NUL
- Studios, publishers, games, manuals, reviews, hardware, and release chronology

Do not blur these layers in technical error reports, package versions, or licenses.

---

## 2. Historical period

The fictional DRX-90 software era runs from:

```text
05.10.1993 through 31.12.1999
```

Dates are written:

```text
DD.MM.YYYY
```

Future games may be assigned any date inside this range, regardless of the order in which they are added to the real project.

The machine displays the user’s real current local date and time. If the date is after 31.12.1999, R/OS may warn that the system is operating beyond its certified period.

Example:

```text
SYSTEM CLOCK ........ OUTSIDE WARRANTY PERIOD
CURRENT DATE ........ 25.07.2026
CONTINUING AT USER'S RISK
```

---

## 3. Reçica Computer Works

### Canonical name

**Reçica Computer Works**, abbreviated **RCW**.

### Role

- designer and manufacturer of the DRX-90;
- publisher of selected flagship software;
- distributor of bundled titles;
- owner of R/OS and AfterHours;
- operator of the fictional software catalog and certification program.

### Character

RCW presents itself as:

- serious;
- technically ambitious;
- slightly overconfident;
- bureaucratic;
- proud of compatibility;
- unable to explain NUL.

Its manuals are clear but occasionally defensive.

### Brand voice

```text
COMPATIBILITY IS A FEATURE.
UNDOCUMENTED COMPATIBILITY IS ALSO A FEATURE.
```

RCW should not be a villain. It is a plausible fictional computer company with ordinary institutional blind spots.

---

## 4. DRX-90

### Full name

**DRX-90 Personal Multimedia System**

### Character

An original home computer influenced by several 1990s platforms without copying one.

It supports:

- a terminal-cell vector display environment;
- fictional disk, archive, ROM, and network media;
- the RCW V/A-16 Audio Array;
- R/OS;
- AfterHours;
- optional V-SCAPE software perspective technology.

The exact CPU architecture should remain fictional unless a later technical manual needs a stable answer.

### Typical boot identity

```text
REÇICA COMPUTER WORKS
DRX-90 PERSONAL MULTIMEDIA SYSTEM

R/OS ROM BIOS 3.11
```

### Product mythology

The DRX-90 was marketed as a serious multimedia and communications machine but became remembered for its unusual software catalog and after-hours use.

---

## 5. R/OS

### Name

**R/OS**

Do not expand the acronym unless a future canonical decision defines it. Different manuals may speculate, but no expansion is currently official.

### Character

R/OS is:

- terse;
- formal;
- stable;
- bureaucratic;
- occasionally facetious;
- more aware of the machine’s age than it admits.

R/OS communicates in uppercase diagnostics and concise help text.

### Current fictional version

Working presentation version:

```text
R/OS 3.11
```

This is not the real project version.

### Commands

Canonical command set may include:

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

R/OS operates on a virtual curated filesystem, not the user’s real files.

---

## 6. AfterHours

### Name

**AfterHours Entertainment Archive**

Usually displayed as **AfterHours**.

### Role

- game launcher;
- software catalog;
- manual archive;
- score viewer;
- publisher and studio index;
- release chronology;
- shareware archive.

### Tone

AfterHours is more colorful and welcoming than the R/OS shell but still belongs to the same machine.

### Catalog categories

- Featured Software
- Shareware and Cover-Disk Archive
- All Software
- Genre Index
- Studio Index
- Release Timeline
- Recently Played
- System Files

---

## 7. NUL

### Canonical expansion

**NUL — Nonessential Utility Layer**

### Official description

> A low-priority resident process providing nonessential maintenance and interface assistance.

### Personality

NUL is:

- funny in a dry way;
- helpful;
- slightly insubordinate;
- observant;
- not constantly present;
- apparently more knowledgeable than R/OS;
- not malicious;
- not a modern conversational assistant;
- not explicitly alive.

### Visual form

A small cursor-like presence:

```text
[_]
```

or a minimal block cursor.

### Introduction

First sessions show only hints:

```text
UNACCOUNTED RESIDENT PROCESS ........ 1
```

Later:

```text
R/OS: NONESSENTIAL UTILITY LAYER LOADED.
NUL: “NONESSENTIAL” WAS ADDED BY MANAGEMENT.
```

### Unofficial expansions

These may appear as jokes, edits, or rumors:

- Not Usually Listening
- Network Utility Lurker
- Nearly Useful Logic
- No User Left

They are not canonical expansions.

### Restrictions

NUL must not:

- claim access to real private data;
- constantly interrupt gameplay;
- become a tutorial mascot;
- speak in internet slang;
- explain the entire mystery;
- become frightening or threatening.

---

## 8. Fictional companies and studios

Personal and geographic source material is transformed into fictional brands.

### Vranidoll Signal Works

Abbreviation: VSW

Role:

- foundational display and signal technology;
- V-SCAPE perspective system;
- signal-processing routines;
- low-level engineering support.

Personality:

- technically respected;
- secretive;
- older than several publishers;
- responsible for systems other studios use without fully understanding.

### Noah Arc Labs

Abbreviation: NAL

Role:

- experimental prototypes;
- vehicle logic;
- unusual algorithms;
- hidden features.

Personality:

- ambitious;
- inventive;
- frequently ships ideas before manuals are ready.

### Sara Circuitworks

Abbreviation: SCW

Role:

- polished mainstream publisher and developer;
- commercial arcade presentation;
- Signal Stack publisher;
- Relay Breaker developer.

Personality:

- professional;
- melodic;
- reliable;
- excellent at turning technical games into products.

### Nora Nova Interactive

Abbreviation: NNI

Role:

- experimental games;
- Mnemonic Nullway developer;
- surreal presentation;
- memory and archive themes.

Personality:

- artistic;
- strange;
- respected;
- sometimes accused of hiding meaning in technical limitations.

### Battenberg Byteforge

Abbreviation: BBF

Role:

- ambitious software;
- performance experiments;
- vehicle design;
- delayed or canceled projects.

Personality:

- brilliant;
- overengineered;
- rarely on schedule.

### Frankenberg Logic Bureau

Abbreviation: FLB

Role:

- puzzle and strategy;
- Signal Stack developer;
- Bureau 9 verification;
- formal rules and mathematical systems.

Personality:

- precise;
- bureaucratic;
- convinced that all problems can be classified.

### Prishtina Vector House

Abbreviation: PVH

Role:

- major publisher;
- vivid vector presentation;
- Bureau 9 publisher;
- Mnemonic Nullway publisher;
- competitor and occasional partner to RCW.

Personality:

- confident;
- culturally influential;
- commercially aggressive;
- willing to publish experimental work.

### Kosova Raster Union

Abbreviation: KRU

Role:

- collective, standards group, or regional technical association;
- may appear in magazines, compatibility programs, or future games.

Personality:

- collaborative;
- standards-focused;
- politically complicated only in the fictional software-industry sense.

Do not map it to real political claims.

### Nürnberg Night Systems

Abbreviation: NNS

Role:

- high-speed nocturnal software;
- Afterline 99 developer;
- advanced V-SCAPE usage.

Personality:

- stylish;
- technically disciplined;
- obsessed with performance after dark.

### Hofheim Home Software

Abbreviation: HHS

Role:

- home software;
- modest utilities;
- Bureau 9 developer;
- Hazard Registry developer;
- cover-disk products.

Personality:

- warm;
- practical;
- underestimated;
- capable of producing enduring software with small teams.

---

## 9. Company relationships

The fictional industry should feel interconnected.

Canonical or strongly preferred relationships:

- RCW manufactures the DRX-90 and publishes selected titles.
- Vranidoll Signal Works licenses foundational technology.
- Nürnberg Night Systems licenses V-SCAPE 3.7 for Afterline 99.
- Nora Nova Interactive uses V-SCAPE 2.2 for Mnemonic Nullway.
- Noah Arc Labs contributes experimental logic to both projection titles.
- Prishtina Vector House competes with RCW but publishes software for its platform.
- Sara Circuitworks publishes the commercially polished Signal Stack.
- Frankenberg Logic Bureau verifies Bureau 9 after its original home-software release.
- Battenberg Byteforge appears in vehicle design, unreleased prototypes, and delayed projects.
- Hofheim Home Software produces software that later receives more prestigious reissues.

Possible future lore:

- studio acquisitions;
- disputed engine credits;
- incompatible expansion modules;
- delayed holiday releases;
- magazine review conflicts;
- canceled software;
- rediscovered development builds;
- shared employees only as fictional roles, never as real biographies.

---

## 10. Fictional technologies

### RCW V/A-16 Audio Array

Hybrid audio system capable of:

- PC-speaker-like tones;
- FM-style synthesis;
- wavetable-like tones;
- low-resolution samples;
- filtered noise.

Its flexible fiction allows 1993–1999 music styles.

### V-SCAPE Perspective System

Developed by Vranidoll Signal Works.

Versions:

```text
V-SCAPE 1.0 — 1995
Experimental corridor projection

V-SCAPE 2.2 — 1997
Used by Mnemonic Nullway

V-SCAPE 3.7 — 1998
Used by Afterline 99
```

It supports pseudo-3D road and corridor rendering inside the DRX-90 display.

### AfterHours media

Games may be presented as loading from:

- floppy images;
- optical data discs;
- ROM modules;
- memory cards;
- archive cartridges;
- network media.

Media should reflect fictional year and studio character.

---

## 11. Canonical software catalog

## 11.1 Loopback

```text
Title: Loopback
Release: 19.12.1993
Classification: Bonus / bundled software
Developer: Noah Arc Labs
Distributor: Reçica Computer Works
Genre: Grid arcade
```

Premise:

A network-routing utility that became a game.

Modes:

- Quick Circuit
- Open Loop

Signature twist:

Paired network ports redirect the route.

Presentation:

Fast cover-disk or bundled-software startup.

## 11.2 Bureau 9

```text
Title: Bureau 9
Subtitle: Numerical Compliance and Grid Analysis
Release: 08.02.1994
Classification: Flagship
Developer: Hofheim Home Software
Publisher: Prishtina Vector House
Verification: Frankenberg Logic Bureau
Genre: Logic
```

Premise:

Sudoku puzzles presented as recovered numerical case files.

Tone:

Calm, institutional, archival.

## 11.3 Hazard Registry

```text
Title: Hazard Registry
Release: 27.05.1994
Classification: Bonus
Developer: Hofheim Home Software
Publisher: Frankenberg Logic Bureau
Genre: Logic and deduction
```

Premise:

Inspect unstable sectors without triggering hidden hazards.

Signature twist:

Some sectors update after a defined number of moves.

## 11.4 Signal Stack

```text
Title: Signal Stack
Subtitle: High-Density Transmission Alignment System
Release: 21.11.1995
Classification: Flagship
Fictional version: 1.4
Developer: Frankenberg Logic Bureau
Publisher: Sara Circuitworks
Technology: Vranidoll Signal Works
Genre: Falling-block puzzle
```

Premise:

Align signal packets and clear channels before the transmission matrix saturates.

Modes:

- Standard Transmission
- Burst Calibration
- Transmission Repair

Key terminology:

| Common concept | Signal Stack term |
|---|---|
| Piece | Signal packet |
| Line | Channel |
| Level | Transmission rate |
| Combo | Signal chain |
| Back-to-back | Sustained transmission |
| Perfect clear | Zero-state matrix |
| Spin clear | Phase rotation |

## 11.5 Relay Breaker

```text
Title: Relay Breaker
Release: 11.08.1996
Classification: Bonus
Developer: Sara Circuitworks
Publisher: Prishtina Vector House
Genre: Ball-and-paddle arcade
```

Premise:

Break connected relay circuits to rebuild a damaged transmission field.

Modes:

- Signal Campaign
- Endless Relay

Signature twist:

Connected blocks charge temporary equipment.

## 11.6 Mnemonic Nullway

```text
Title: Mnemonic Nullway
Release: 06.06.1997
Classification: Flagship
Fictional version: 2.2
Developer: Nora Nova Interactive
Publisher: Prishtina Vector House
Technology: V-SCAPE 2.2
Additional technology: Noah Arc Labs
Genre: Perspective runner
Catalog: PVH-AH-271
```

Premise:

REC-0, an unauthorized recovery process, traverses damaged memory before the archive closes.

Player:

REC-0.

Domains:

- Document Cache
- Entertainment Memory
- Message Spool
- System Core
- Unindexed Space

Signature mechanic:

Phase shifting between visible and archived memory.

## 11.7 Afterline 99

```text
Title: Afterline 99
Release: 17.09.1998
Classification: Flagship
Fictional version: 3.7
Developer: Nürnberg Night Systems
Publisher: Reçica Computer Works
Technology: V-SCAPE 3.7
Signal simulation: Vranidoll Signal Works
Vehicle logic: Noah Arc Labs
Genre: Perspective racing
Catalog: RCW-AH-399
```

Premise:

Race unauthorized signal routes after the network curfew.

Principal routes:

- Static Coast
- Glass Relay
- Vranidoll Divide
- Prishtina Midnight Exchange
- Null Meridian

Signal craft:

- SC-1 Courier
- Battenberg B4
- Nora Vector
- Frankenberg Heavy Relay
- Noah Arc Prototype
- RCW Nightline

## 11.8 Packet Sweep

```text
Title: Packet Sweep
Release: Unknown
Classification: Hidden
Attributed developer: Frankenberg Logic Bureau
Genre: Miniature action game
```

Premise:

A maintenance cursor collects valid packets and avoids checksum errors.

Rules:

- absent from public catalog;
- absent from normal website game list;
- referenced only through subtle traces;
- small but complete.

---

## 12. Software release chronology

| Date | Software |
|---|---|
| 05.10.1993 | Beginning of certified DRX-90 commercial period |
| 19.12.1993 | Loopback |
| 08.02.1994 | Bureau 9 |
| 27.05.1994 | Hazard Registry |
| 21.11.1995 | Signal Stack |
| 11.08.1996 | Relay Breaker |
| 06.06.1997 | Mnemonic Nullway |
| 17.09.1998 | Afterline 99 |
| 31.12.1999 | End of certified DRX-90 software period |

Packet Sweep has no reliable release date.

---

## 13. Cross-game references

Cross-references should be occasional and optional.

Examples:

- Afterline 99 billboard advertises Signal Stack.
- Bureau 9 case IDs reference fictional catalog numbers.
- Mnemonic Nullway contains damaged title fragments from other games.
- Manuals disagree about who invented a rendering technique.
- V-SCAPE version history appears in several products.
- A track vehicle is credited to Battenberg Byteforge.
- NUL comments on files from other studios.
- A fictional review mentions an unreleased game.

Do not require players to understand references to play.

---

## 14. Fictional filesystem

Suggested structure:

```text
R:\
├── SYSTEM\
│   ├── README.TXT
│   ├── DRIVERS.LOG
│   └── WARRANTY.DAT
├── AFTERHRS\
│   ├── GAMES\
│   ├── MANUALS\
│   └── SCORES\
├── ARCHIVE\
│   ├── STUDIOS\
│   ├── REVIEWS\
│   └── RECOVERED\
└── TEMP\
```

Content types:

- manuals;
- reviews;
- studio profiles;
- technical notes;
- warranty notices;
- canceled game references;
- corrupted fragments;
- NUL annotations.

The virtual filesystem must never expose real user files.

---

## 15. Tone guide

### Core tone

- technical;
- nocturnal;
- nostalgic;
- sincere;
- dryly facetious;
- occasionally uncanny;
- never grim for long.

### Humor examples

Good:

```text
USER PATIENCE ........ UNVERIFIED
CONTINUING WITHOUT IT
```

```text
NUL: THIS FILE IS NOT CORRUPTED.
R/OS: FILE STATUS: CORRUPTED.
NUL: WE DISAGREE.
```

Bad:

- modern memes;
- emoji jokes;
- long sarcastic monologues;
- fake malware behavior;
- threats;
- fake access to private photos, files, or browsing history;
- constant fourth-wall references to Rust or AI.

### Melancholy

Allowed themes:

- expired warranties;
- abandoned studios;
- unreleased games;
- archives closing;
- obsolete systems remembering activity;
- contradictory records;
- software outliving its certified period.

Avoid graphic horror, jump scares, or claims of surveillance.

---

## 16. Privacy-safe canon rules

The following names draw inspiration from real places or family names:

- Vranidoll
- Noah
- Sara
- Nora
- Battenberg
- Frankenberg
- Prishtina
- Kosovo/Kosova
- Nürnberg
- Hofheim
- Reçica

Use them as:

- fictional companies;
- products;
- routes;
- systems;
- abstract locations.

Do not add:

- “named after the creator’s child”;
- ancestor narratives;
- real addresses;
- exact childhood events;
- schools or employers;
- claims that the fictional people are real family members;
- real political history;
- sensitive location detail.

The fiction should stand independently.

---

## 17. Canon change process

A canon change should:

1. update this document;
2. update affected manuals and metadata;
3. update tests or content validation;
4. add a decision entry if significant;
5. avoid retconning released content without explanation.

When uncertain, leave a detail unspecified rather than inventing an answer.

---

## 18. Canon checklist for new content

- [ ] Name matches exact spelling.
- [ ] Date uses `DD.MM.YYYY`.
- [ ] Date falls within fictional period unless intentionally marked unknown.
- [ ] Studio and publisher relationship is plausible.
- [ ] Tone is sincere with restrained humor.
- [ ] No real private biography is implied.
- [ ] NUL is used sparingly.
- [ ] Reference is optional for gameplay understanding.
- [ ] Technical terms match existing fiction.
- [ ] Real project version and fictional version are not confused.
