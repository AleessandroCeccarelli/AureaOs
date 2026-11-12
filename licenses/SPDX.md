<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# SPDX Summary for AureaOS

This document summarizes the licensing status of original AureaOS work and reviewed third-party content. It complements `LICENSE` and `THIRD_PARTY_NOTICES.md`.

## Project target

- Project: AureaOS
- Repository: AleessandroCeccarelli/AureaOs
- Primary project license: GPL-3.0-or-later
- License file: `LICENSE`

## Original AureaOS content

- SPDX-License-Identifier: GPL-3.0-or-later
- Scope: project-authored source, documentation, scripts, and design-system material unless a file states otherwise.

## Bundled third-party content

- Microsoft Fluent UI System Icons — MIT; curated reference subset at `aurea-human-interfaces/symbols/source/fluent-ui`.
- Google Noto Sans, Noto Color Emoji, Noto Sans Symbols 2, and Noto Sans Math — OFL-1.1; temporary font and glyph-coverage sources under `aurea-human-interfaces/fonts/source/google-fonts`.
- Adobe Source Code Pro — OFL-1.1; temporary monospaced fallback at `aurea-human-interfaces/fonts/source/adobe/source-code-pro`.
- Debian packages — licenses vary by package; review through package metadata and release records.

## Notes

- The repository is not a single-license bundle when third-party components are redistributed.
- Original AureaOS work remains under GPL-3.0-or-later. Third-party components retain their own licenses.
- First-party product identity and design control do not imply a proprietary or dual-licensing exception.
- Update this summary and `THIRD_PARTY_NOTICES.md` whenever third-party material is added, removed, or materially changed.

## Review status

- Project `LICENSE`: reviewed.
- Retained Fluent UI reference subset: license file retained; MIT attribution recorded.
- Retained font sources: upstream license files retained; OFL-1.1 recorded.
- Debian base dependencies: package-level review required for release bundles.
