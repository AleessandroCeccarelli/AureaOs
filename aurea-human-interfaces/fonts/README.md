<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Typography

This directory contains the approved direction and the limited set of temporary font sources used during development.

## Font direction

AureaOS is developing an original type family under the name **Aurea Sans**. It is intended to become the primary typeface for the desktop and first-party applications after validation of originality, language coverage, rendering, accessibility, metadata, and licensing.

Aurea Sans is not yet an approved production system font. Until it is ready, the retained third-party families are development and coverage fallbacks only; they are not the final AureaOS typography identity.

## Retained development support

- **Noto Sans** — broad-coverage temporary sans-serif fallback.
- **Source Code Pro** — temporary monospaced fallback for terminal and developer interfaces.
- **Noto Color Emoji** — temporary emoji glyph coverage while the AureaOS emoji direction is designed.
- **Noto Sans Symbols 2** — supplementary Unicode symbol coverage.
- **Noto Sans Math** — mathematical glyph coverage.

These sources are deliberately limited to distinct coverage needs. Redundant general-purpose sans-serif families and duplicate font families have been removed.

## Product policy

The shipped desktop and first-party applications should use AureaOS-approved and certified font assets. Third-party font installation and arbitrary external font stacks are outside the supported product customization model.

The Aurea Sans project is intended to use the **GPL-3.0-or-later** license. Its metadata must identify AureaOS and the relevant family and style information. Final font files must be reviewed before approval for system use.

## Design intent

Typography should be readable, clean, modern, calm, and consistent across shell, applications, and settings. Prefer comfort and legibility over ornamental styling.

## Licensing

Third-party font sources retain their upstream licenses. Their notices are recorded in `../../licenses/THIRD_PARTY_NOTICES.md`.
