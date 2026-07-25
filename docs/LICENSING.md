# Raster Nights Licensing and Identity Policy

**Status:** Working policy; obtain qualified legal review before relying on it for major commercial or trademark decisions  
**Audience:** Owner, coding agents, contributors, redistributors, and reviewers

This document separates the rights and rules for software source code, technical documentation, creative assets, fictional content, and project identity.

The goal is to keep the software genuinely open source while reserving the official Raster Nights identity, fictional universe, game titles, logos, artwork, music, and narrative content.

---

## 1. Intended licensing model

### Software source code

Intended license:

```text
Mozilla Public License 2.0
SPDX-License-Identifier: MPL-2.0
```

This includes original:

- Rust source;
- TypeScript source;
- JavaScript source;
- build scripts;
- source code for tooling;
- source code for tests.

The final repository must include the full MPL-2.0 license text before public release.

### General technical documentation

Intended license:

```text
Creative Commons Attribution 4.0 International
CC BY 4.0
```

This may include:

- architecture explanations;
- development instructions;
- generic technical diagrams;
- testing guidance.

Documents that contain substantial fictional narrative or reserved branding may be excluded or dual-labeled more restrictively.

### Reserved creative material

Unless an explicit file header or asset manifest says otherwise, reserve:

- Raster Nights name and logo;
- Reçica Computer Works identity;
- DRX-90 identity;
- R/OS identity;
- AfterHours identity;
- NUL identity;
- fictional studio names and logos;
- game titles;
- fictional character and vehicle names;
- fictional manuals and narrative writing;
- lore and canon;
- artwork;
- music;
- sound effects;
- title treatments;
- marketing illustrations;
- product catalog design.

These materials should be treated as all rights reserved unless separately licensed.

### Third-party material

Third-party fonts, libraries, icons, tools, or assets remain under their original licenses. Their notices must be preserved.

---

## 2. Why MPL-2.0 is the working software choice

The project owner wants:

- public source;
- reciprocal sharing of modifications to covered source files;
- fewer barriers than strong whole-program copyleft;
- compatibility with a mixed repository containing separately reserved creative content.

MPL-2.0 is file-level copyleft. It is intended to require modifications to MPL-covered files to remain available under the MPL while permitting combination with separately licensed files.

This is a product and project-governance recommendation, not legal advice.

### Why not MIT or Apache-2.0 as default

A permissive license would allow modified source to be incorporated into proprietary derivatives without requiring modifications to the covered code to be published.

That may be acceptable for some projects but is weaker than the current owner preference.

### Why not GPL-3.0 as default

GPL applies broader copyleft to the combined work and may create more friction for reuse, embedding, or mixed licensing.

### Why not AGPL-3.0 as default

AGPL is most relevant when modified software is operated as a network service and remote users should receive source-offer rights.

Raster Nights v1:

- has no backend;
- has no hosted official service;
- has no accounts;
- makes no outbound requests;
- runs locally or as static browser code.

The additional AGPL network provision does not solve the project’s primary concern.

---

## 3. Code file headers

Where file headers are used:

```text
// SPDX-License-Identifier: MPL-2.0
```

Do not add long boilerplate headers to every file unless required. An SPDX identifier plus repository license and notice is generally clearer.

Generated files should state:

- whether they are generated;
- their source;
- whether they are covered by the same license;
- whether manual edits are allowed.

---

## 4. Asset manifest

The repository should include a machine-readable or Markdown asset manifest before public release.

Recommended fields:

```text
path
title
creator
source
license
copyright holder
modification status
required attribution
runtime or source asset
```

Example:

```text
assets/fonts/example.woff2
License: SIL Open Font License 1.1
Source: <documented upstream project>
Modified: No
```

Never add an asset because it is described vaguely as “free.” Confirm the exact license.

---

## 5. Reserved names and fork policy

Forks may exercise rights granted by the software license, but they must not imply that they are official Raster Nights releases.

Reserved official identity includes:

- Raster Nights;
- Reçica Computer Works;
- DRX-90;
- R/OS;
- AfterHours;
- NUL;
- fictional studios;
- official game names;
- logos and title marks;
- official website appearance where it functions as source identification.

### Required fork behavior

A redistributed modified fork should:

- use a different project name;
- use a different executable name;
- replace official logos;
- remove or replace reserved fictional identity and creative assets unless separately permitted;
- clearly state that it is unofficial;
- preserve notices required by the software license;
- avoid implying endorsement by Drilon Reçica.

### Unmodified redistribution

The final asset policy may allow redistribution of unmodified official release archives solely so users can install and run Raster Nights. This permission should be written explicitly before public release.

### Source builds

Users must be able to build and run the official source. Therefore, the repository needs a narrow license or permission for necessary bundled runtime assets.

A practical final policy might distinguish:

1. permission to use bundled assets while running an unmodified official build;
2. no permission to use those assets in branded derivative projects;
3. no permission to sell or rebrand the fictional identity without consent.

This section requires careful final legal drafting.

---

## 6. Fictional content in source repositories

Files containing fictional manuals, lore, reviews, studio profiles, title copy, and game narrative are not automatically software code merely because they are stored in Git.

Place a notice at the top of relevant directories, for example:

```text
The fictional names, narrative content, manuals, logos, artwork, music,
and other creative assets in this directory are not licensed under
MPL-2.0 unless an individual file explicitly says otherwise.
```

Coding agents must not assume repository-wide MPL coverage.

---

## 7. Generated content and AI-assisted work

AI-assisted code and content must still be reviewed for:

- originality;
- accidental copying;
- license compatibility;
- quality;
- consistency;
- provenance of any referenced assets.

Do not instruct agents to imitate a living artist, a specific commercial game’s protected presentation, or copyrighted music.

Do not include generated output that contains recognizable third-party characters, logos, music, or extensive text.

Maintain human responsibility for final selection and publication.

---

## 8. Dependency licenses

Before adding a dependency:

1. identify its license;
2. confirm compatibility with MPL-2.0 distribution;
3. record required notices;
4. disable unnecessary features;
5. ensure it does not bring prohibited telemetry or network behavior;
6. preserve attribution files in release archives when required.

The release process should generate or include a dependency license report if this remains simple.

Do not introduce a complicated compliance service. A checked-in notices file or build-generated report is sufficient.

---

## 9. Font licensing

The browser requires a bundled open-source monospace or bitmap-style font.

Before adding one:

- verify redistribution permission;
- verify web embedding permission;
- preserve the full font license;
- do not redistribute editor/source font files that are not required or permitted;
- do not share or package proprietary system fonts.

The application must not require users to install a proprietary font.

---

## 10. Music and audio

Original music and sound effects should have documented creators and rights.

For each asset record:

- author;
- source project;
- date;
- license;
- whether source/tracker file is included;
- whether samples are original or third-party.

Do not use unverified sample packs.

If a contributor creates music or art, obtain a written contribution agreement or explicit license suitable for the intended reserved-content model.

---

## 11. External contributions

Maintenance pull requests may be accepted.

Before merging an external contribution, confirm:

- contributor has authority to submit it;
- code can be licensed under MPL-2.0;
- no copied game assets or code are included;
- no new fictional canon is added without owner approval;
- no dependency introduces incompatible terms;
- attribution requirements are understood.

The repository may use a lightweight Developer Certificate of Origin process later, but should not introduce legal bureaucracy until contributions make it necessary.

---

## 12. Commercial use

MPL-2.0 generally permits commercial use of covered software subject to its terms.

The reserved creative identity is separate.

A third party may be able to use the software code under the license but may not use reserved Raster Nights branding and assets without permission.

The owner may publish official builds, merchandise, soundtracks, or commercial editions without changing the open-source status of covered code.

Obtain legal advice before relying on this section for a business transaction.

---

## 13. Trademark strategy

Even before formal registration, use the marks consistently:

- Raster Nights
- Reçica Computer Works
- DRX-90
- R/OS
- AfterHours

A future trademark policy may explain:

- nominative references;
- screenshots and reviews;
- community discussion;
- unofficial forks;
- merchandise;
- domain names;
- logo use.

Do not add ™ or ® indiscriminately. `®` must not be used without registration in the relevant jurisdiction.

---

## 14. Required repository files before public release

The final repository should contain:

```text
LICENSE                  MPL-2.0 full text
NOTICE                   project and third-party notices
ASSET-LICENSES.md        asset-by-asset licensing
TRADEMARKS.md            fork and identity policy
docs/LICENSING.md        explanatory policy
```

Only `docs/LICENSING.md` is included in this specification package. The legal files should be created after the policy is finalized.

---

## 15. Release archive requirements

Each official release archive should include:

- executable;
- README or short usage document;
- MPL-2.0 license;
- notices;
- third-party licenses required for redistribution;
- asset notices where required;
- version information;
- checksum published separately or alongside the archive.

No signing or notarization is planned.

---

## 16. Licensing checklist for coding agents

Before adding code or content:

- [ ] Is this code, documentation, or creative content?
- [ ] Which license applies?
- [ ] Is there third-party material?
- [ ] Is attribution required?
- [ ] Does a bundled asset have a verified license?
- [ ] Does the change use a reserved project name appropriately?
- [ ] Would a fork need to replace this item?
- [ ] Has AI-generated output been reviewed for copying risk?
- [ ] Are notices updated?
- [ ] Is legal review needed before publication?

---

## 17. Final warning

Do not make legal claims beyond the repository’s actual license text.

This document expresses intended project policy. The final published license files and qualified legal advice govern where they differ.
