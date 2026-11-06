# SPDX Summary for AureaOS

This file provides a simple SPDX-style summary of the licensing status for the AureaOS project and its reviewed third-party content.

This document supports project review and auditing. It does not replace the project license file or the detailed third-party registry in `THIRD_PARTY_NOTICES.md`.

## Project target

- Project: AureaOS
- Repository: AleessandroCeccarelli/AureaOs
- Primary project license: GPL-3.0-or-later
- License file: `LICENSE`

## SPDX overview

SPDX-License-Identifier: GPL-3.0-or-later

### Original AureaOS project content

- License: GPL-3.0-or-later
- Scope: original project source files, documentation, scripts, and project-authored content unless otherwise stated
- Reference: `LICENSE`

### Bundled or reviewed third-party content

- Microsoft Fluent UI Symbols
  - SPDX-License-Identifier: MIT
  - Source: `aurea-human-interfaces/symbols/source/fluent-ui`
  - Notes: Included as source design assets. Review status: reviewed.

- Google Noto Color Emoji
  - SPDX-License-Identifier: OFL-1.1
  - Source: `aurea-human-interfaces/fonts/source/google-fonts/noto-color-emoji`
  - Notes: Included as a font source package. Review status: reviewed.

- Debian package ecosystem
  - SPDX-License-Identifier: varies by package and license
  - Source: Debian package repositories
  - Notes: Runtime dependencies are managed through Debian package metadata and system package policy. They are not all vendored in this repository and must be reviewed at build and release time.

## Notes

- The project as a whole is not a single license bundle if third-party components with different licenses are redistributed.
- The project license applies to the original AureaOS work unless a specific file states otherwise.
- Third-party components keep their own licenses and must be tracked in `THIRD_PARTY_NOTICES.md`.
- This summary should be updated whenever a new dependency, font, icon set, asset, or vendored library is introduced.

## Current review status

- `LICENSE` (project): reviewed
- `aurea-human-interfaces/symbols/source/fluent-ui`: reviewed
- `aurea-human-interfaces/fonts/source/google-fonts/noto-color-emoji`: reviewed
- Debian base dependencies: pending package-level review as the OS matures

## Outstanding verification items

The following are known to require confirmation when the corresponding resources are used in release builds or distribution bundles:

- exact package-origin provenance for any Debian packages included in project bundles
- any additional third-party font or icon assets newly added in the future
- any third-party image, wallpaper, or artwork not yet listed in `THIRD_PARTY_NOTICES.md`
- any proprietary or external runtime components that might be introduced later
