# SPDX Summary for AureaOS

This file provides a simple SPDX-style summary of the licensing status for the AureaOS project and its reviewed third-party content.

This document is intended to support project review, auditing, and maintenance as the project grows. It is not a replacement for the project-level license files or for the detailed third-party registry in `THIRD_PARTY_NOTICES.md`.

## Project target

- Project: AureaOS
- Repository: AleessandroCeccarelli/AureaOs
- Primary project license: GPL-3.0-only
- License file: `LICENSE`

## SPDX overview

SPDX-License-Identifier: GPL-3.0-only

### Original AureaOS project content

- License: GPL-3.0-only
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
  - Notes: Runtime dependencies are managed through Debian package metadata and system package policy. They are not all vendored in this repository and must be reviewed at build time and release time.

## Notes

- The project as a whole is not "GPL-3.0-only" in a literal sense if third-party components with different licenses are bundled or redistributed.
- The project license applies to the original AureaOS work unless another file explicitly overrides it.
- Third-party components keep their own licenses and must be tracked in `THIRD_PARTY_NOTICES.md`.
- This summary should be updated whenever a new dependency is introduced or when a reviewed component changes license status.

## Recommended maintenance rule

Whenever a new dependency, font, icon pack, asset, or vendored library is added to the project, do the following:

1. Add a record to `licenses/THIRD_PARTY_NOTICES.md`
2. Add the relevant SPDX-style identifier here if appropriate
3. Confirm the license is compatible with the project goals and distribution model
4. Keep the path and purpose clear for future audit work

## Current review status

- `LICENSE` (project): reviewed
- `aurea-human-interfaces/symbols/source/fluent-ui` : reviewed
- `aurea-human-interfaces/fonts/source/google-fonts/noto-color-emoji` : reviewed
- Debian base dependencies: pending package-level review as the OS matures

## Example future entry format

```text
- Component Name
  - SPDX-License-Identifier: MIT
  - Source: path/to/component
  - Notes: short description and review status
```

This file is intentionally simple and intentionally maintainable.
