<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# AureaOS Typography

Owns font sources, licensing records, typography decisions and the future original Aurea Sans family.

## Structure

- `source/google-fonts/` — retained Noto families and upstream notices.
- `source/adobe/` — retained Source Code Pro and its upstream notice.
- `generated/` — reserved for original font build outputs, subsets and validated runtime packages.

## Current development support

| Family | Role | License |
|---|---|---|
| Noto Sans | Temporary general UI fallback | SIL Open Font License 1.1 |
| Noto Color Emoji | Temporary emoji coverage | SIL Open Font License 1.1 |
| Noto Sans Symbols 2 | Supplementary Unicode coverage | SIL Open Font License 1.1 |
| Noto Sans Math | Mathematical glyph coverage | SIL Open Font License 1.1 |
| Source Code Pro | Temporary monospaced/developer fallback | SIL Open Font License 1.1 |

Keep upstream notices beside each family. These are not the final AureaOS typography identity.

## Destination: Aurea Sans

Aurea Sans is an intended original project type family, not an existing approved production font. Planned work: typography brief, glyph design, spacing/kerning, language coverage, regular and emphasized styles, shaping and hinting validation, readability and display-scale tests, metadata and license review, reproducible font build and runtime packaging.

Do not rename or relicense third-party fonts as Aurea Sans. Do not select the final system font stack until the original family is approved and fallback coverage is tested.
